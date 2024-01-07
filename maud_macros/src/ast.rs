use std::fmt::{self, Display, Formatter};

use proc_macro2::TokenStream;
use proc_macro2_diagnostics::{Diagnostic, SpanDiagnosticExt};
use quote::ToTokens;
use syn::{
    braced, bracketed,
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::{Pair, Punctuated},
    spanned::Spanned,
    token::{Brace, Bracket, Paren},
    Error, Expr, Ident, Lit, LitInt, LitStr, Local, Pat, Stmt, Token,
};

#[derive(Debug, Clone)]
pub struct Markups {
    pub markups: Vec<Markup>,
}

impl Parse for Markups {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut markups = Vec::new();
        while !input.is_empty() {
            markups.push(input.call(Markup::parse_with_let)?);
        }
        Ok(Self { markups })
    }
}

impl ToTokens for Markups {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for markup in &self.markups {
            markup.to_tokens(tokens);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Markup {
    Block(Block),
    Lit(HtmlLit),
    Splice { paren_token: Paren, expr: Expr },
    Element(Element),
    ControlFlow(ControlFlow),
    Semi(Token![;]),
}

impl Markup {
    pub fn parse_with_let(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![let])
            || input.peek(Token![if])
            || input.peek(Token![for])
            || input.peek(Token![while])
            || input.peek(Token![match])
        {
            let kw = input.call(Ident::parse_any)?;
            return Err(kw
                .span()
                .error(format!("found keyword `{kw}`"))
                .help(format!("should this be a `@{kw}`?"))
                .into());
        }

        let lookahead = input.lookahead1();

        if lookahead.peek(Brace) {
            input.parse().map(Self::Block)
        } else if lookahead.peek(LitStr) || lookahead.peek(LitInt) {
            input.parse().map(Self::Lit)
        } else if lookahead.peek(Paren) {
            let content;
            Ok(Self::Splice {
                paren_token: parenthesized!(content in input),
                expr: content.parse()?,
            })
        } else if lookahead.peek(Ident::peek_any)
            || lookahead.peek(Token![.])
            || lookahead.peek(Token![#])
        {
            input.parse().map(Self::Element)
        } else if lookahead.peek(Token![@]) {
            input.parse().map(Self::ControlFlow)
        } else if lookahead.peek(Token![;]) {
            input.parse().map(Self::Semi)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Markup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let markup = Self::parse_with_let(input)?;

        if let Self::ControlFlow(ControlFlow {
            kind: ControlFlowKind::Let(expr),
            ..
        }) = &markup
        {
            Err(Error::new_spanned(
                expr,
                "`@let` bindings are only allowed inside blocks",
            ))
        } else {
            Ok(markup)
        }
    }
}

impl ToTokens for Markup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Block(block) => block.to_tokens(tokens),
            Self::Lit(lit) => lit.to_tokens(tokens),
            Self::Splice { paren_token, expr } => {
                paren_token.surround(tokens, |tokens| {
                    expr.to_tokens(tokens);
                });
            }
            Self::Element(element) => element.to_tokens(tokens),
            Self::ControlFlow(control_flow) => control_flow.to_tokens(tokens),
            Self::Semi(semi) => semi.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: Option<HtmlName>,
    pub attrs: Vec<Attribute>,
    pub body: ElementBody,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: if input.peek(Ident::peek_any) {
                Some(input.parse()?)
            } else {
                None
            },
            attrs: {
                let mut id_pushed = false;
                let mut attrs = Vec::new();

                while input.peek(Ident::peek_any)
                    || input.peek(Lit)
                    || input.peek(Token![.])
                    || input.peek(Token![#])
                {
                    let attr = input.parse()?;

                    if let Attribute::Id { .. } = attr {
                        if id_pushed {
                            return Err(Error::new_spanned(
                                attr,
                                "duplicate id (`#`) attribute specified",
                            ));
                        }
                        id_pushed = true;
                    }

                    attrs.push(attr);
                }

                if input.peek(Token![/]) {
                    return Err(input
                        .span()
                        .error("void elements must use `;`, not `/`")
                        .help("change this to `;`")
                        .help("see https://github.com/lambda-fairy/maud/pull/315 for details")
                        .into());
                }

                if !(input.peek(Brace) || input.peek(Token![;])) {
                    let lookahead = input.lookahead1();

                    lookahead.peek(Ident::peek_any);
                    lookahead.peek(Lit);
                    lookahead.peek(Token![.]);
                    lookahead.peek(Token![#]);
                    lookahead.peek(Brace);
                    lookahead.peek(Token![;]);

                    return Err(lookahead.error());
                }

                attrs
            },
            body: input.parse()?,
        })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(name) = &self.name {
            name.to_tokens(tokens);
        }
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.body.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub enum ElementBody {
    Void(Token![;]),
    Block(Block),
}

impl Parse for ElementBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![;]) {
            input.parse().map(Self::Void)
        } else if lookahead.peek(Brace) {
            input.parse().map(Self::Block)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for ElementBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Void(semi) => semi.to_tokens(tokens),
            Self::Block(block) => block.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub brace_token: Brace,
    pub markups: Markups,
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            brace_token: braced!(content in input),
            markups: content.parse()?,
        })
    }
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brace_token.surround(tokens, |tokens| {
            self.markups.to_tokens(tokens);
        });
    }
}

#[derive(Debug, Clone)]
pub enum Attribute {
    Class {
        dot_token: Token![.],
        name: AttributeName,
        toggler: Option<Toggler>,
    },
    Id {
        pound_token: Token![#],
        name: AttributeName,
    },
    Named {
        name: AttributeName,
        attr_type: AttributeType,
    },
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![.]) {
            Ok(Self::Class {
                dot_token: input.parse()?,
                name: input.parse()?,
                toggler: {
                    let lookahead = input.lookahead1();

                    if lookahead.peek(Bracket) {
                        Some(input.parse()?)
                    } else {
                        None
                    }
                },
            })
        } else if lookahead.peek(Token![#]) {
            Ok(Self::Id {
                pound_token: input.parse()?,
                name: input.parse()?,
            })
        } else {
            Ok(Self::Named {
                name: input.parse()?,
                attr_type: input.parse()?,
            })
        }
    }
}

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Class {
                dot_token,
                name,
                toggler,
            } => {
                dot_token.to_tokens(tokens);
                name.to_tokens(tokens);
                if let Some(toggler) = toggler {
                    toggler.to_tokens(tokens);
                }
            }
            Self::Id { pound_token, name } => {
                pound_token.to_tokens(tokens);
                name.to_tokens(tokens);
            }
            Self::Named { name, attr_type } => {
                name.to_tokens(tokens);
                attr_type.to_tokens(tokens);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AttributeName {
    Normal(HtmlName),
    Markup(Markup),
}

impl Parse for AttributeName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = if input.peek(Ident::peek_any) || input.peek(LitStr) || input.peek(LitInt) {
            input.parse().map(Self::Normal)
        } else {
            input.parse().map(Self::Markup)
        };

        if input.peek(Token![?]) {
            input.parse::<Token![?]>()?;
        }

        name
    }
}

impl ToTokens for AttributeName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Normal(name) => name.to_tokens(tokens),
            Self::Markup(markup) => markup.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AttributeType {
    Normal {
        eq_token: Token![=],
        value: Markup,
    },
    Optional {
        eq_token: Token![=],
        toggler: Toggler,
    },
    Empty(Option<Toggler>),
}

impl Parse for AttributeType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![=]) {
            let eq_token = input.parse()?;

            if input.peek(Bracket) {
                Ok(Self::Optional {
                    eq_token,
                    toggler: input.parse()?,
                })
            } else {
                Ok(Self::Normal {
                    eq_token,
                    value: input.parse()?,
                })
            }
        } else if lookahead.peek(Bracket) {
            Ok(Self::Empty(Some(input.parse()?)))
        } else {
            Ok(Self::Empty(None))
        }
    }
}

impl ToTokens for AttributeType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Normal {
                eq_token,
                value: markup,
            } => {
                eq_token.to_tokens(tokens);
                markup.to_tokens(tokens);
            }
            Self::Optional { eq_token, toggler } => {
                eq_token.to_tokens(tokens);
                toggler.to_tokens(tokens);
            }
            Self::Empty(toggler) => {
                if let Some(toggler) = toggler {
                    toggler.to_tokens(tokens);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct HtmlName {
    pub name: Punctuated<HtmlNameFragment, HtmlNamePunct>,
}

impl Parse for HtmlName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: {
                let mut punctuated = Punctuated::new();

                loop {
                    punctuated.push_value(input.parse()?);

                    if !(input.peek(Token![-]) || input.peek(Token![:])) {
                        break;
                    }

                    let punct = input.parse()?;
                    punctuated.push_punct(punct);
                }

                punctuated
            },
        })
    }
}

impl ToTokens for HtmlName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
    }
}

impl Display for HtmlName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for pair in self.name.pairs() {
            match pair {
                Pair::Punctuated(fragment, punct) => {
                    fragment.fmt(f)?;
                    punct.fmt(f)?;
                }
                Pair::End(fragment) => {
                    fragment.fmt(f)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum HtmlNameFragment {
    Ident(Ident),
    Lit(HtmlLit),
    Empty,
}

impl Parse for HtmlNameFragment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Ident::peek_any) {
            input.call(Ident::parse_any).map(Self::Ident)
        } else if lookahead.peek(LitStr) || lookahead.peek(LitInt) {
            input.parse().map(Self::Lit)
        } else if lookahead.peek(Token![-]) || lookahead.peek(Token![:]) {
            Ok(Self::Empty)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for HtmlNameFragment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Lit(lit) => lit.to_tokens(tokens),
            Self::Empty => {}
        }
    }
}

impl Display for HtmlNameFragment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Ident(ident) => ident.fmt(f),
            Self::Lit(lit) => lit.fmt(f),
            Self::Empty => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HtmlLit {
    Str(LitStr),
    Int(LitInt),
}

impl Parse for HtmlLit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(LitStr) {
            input.parse().map(Self::Str)
        } else if lookahead.peek(LitInt) {
            input.parse().map(Self::Int)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for HtmlLit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Str(lit) => lit.to_tokens(tokens),
            Self::Int(lit) => lit.to_tokens(tokens),
        }
    }
}

impl Display for HtmlLit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Str(lit) => lit.value().fmt(f),
            Self::Int(lit) => lit.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HtmlNamePunct {
    Colon(Token![:]),
    Hyphen(Token![-]),
}

impl Parse for HtmlNamePunct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![:]) {
            input.parse().map(Self::Colon)
        } else if lookahead.peek(Token![-]) {
            input.parse().map(Self::Hyphen)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for HtmlNamePunct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Colon(token) => token.to_tokens(tokens),
            Self::Hyphen(token) => token.to_tokens(tokens),
        }
    }
}

impl Display for HtmlNamePunct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Colon(_) => f.write_str(":"),
            Self::Hyphen(_) => f.write_str("-"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toggler {
    pub bracket_token: Bracket,
    pub cond: Expr,
}

impl Parse for Toggler {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            bracket_token: bracketed!(content in input),
            cond: content.parse()?,
        })
    }
}

impl ToTokens for Toggler {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.bracket_token.surround(tokens, |tokens| {
            self.cond.to_tokens(tokens);
        });
    }
}

#[derive(Debug, Clone)]
pub struct ControlFlow {
    pub at_token: Token![@],
    pub kind: ControlFlowKind,
}

impl Parse for ControlFlow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            at_token: input.parse()?,
            kind: {
                let lookahead = input.lookahead1();

                if lookahead.peek(Token![if]) {
                    ControlFlowKind::If(input.parse()?)
                } else if lookahead.peek(Token![for]) {
                    ControlFlowKind::For(input.parse()?)
                } else if lookahead.peek(Token![while]) {
                    ControlFlowKind::While(input.parse()?)
                } else if lookahead.peek(Token![match]) {
                    ControlFlowKind::Match(input.parse()?)
                } else if lookahead.peek(Token![let]) {
                    let Stmt::Local(local) = input.parse()? else {
                        unreachable!()
                    };

                    ControlFlowKind::Let(local)
                } else {
                    return Err(lookahead.error());
                }
            },
        })
    }
}

impl ToTokens for ControlFlow {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.at_token.to_tokens(tokens);
        match &self.kind {
            ControlFlowKind::Let(local) => local.to_tokens(tokens),
            ControlFlowKind::If(if_) => if_.to_tokens(tokens),
            ControlFlowKind::For(for_) => for_.to_tokens(tokens),
            ControlFlowKind::While(while_) => while_.to_tokens(tokens),
            ControlFlowKind::Match(match_) => match_.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ControlFlowKind {
    Let(Local),
    If(If),
    For(For),
    While(While),
    Match(Match),
}

#[derive(Debug, Clone)]
pub struct If {
    pub if_token: Token![if],
    pub cond: Expr,
    pub then_branch: Block,
    pub else_branch: Option<(Token![@], Token![else], Box<IfOrBlock>)>,
}

impl Parse for If {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            if_token: input.parse()?,
            cond: input.call(Expr::parse_without_eager_brace)?,
            then_branch: input.parse()?,
            else_branch: {
                if input.peek(Token![@]) {
                    Some((input.parse()?, input.parse()?, input.parse()?))
                } else {
                    None
                }
            },
        })
    }
}

impl ToTokens for If {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.if_token.to_tokens(tokens);
        self.cond.to_tokens(tokens);
        self.then_branch.to_tokens(tokens);
        if let Some((at_token, else_token, else_branch)) = &self.else_branch {
            at_token.to_tokens(tokens);
            else_token.to_tokens(tokens);
            else_branch.to_tokens(tokens);
        }
    }
}

#[derive(Debug, Clone)]
pub enum IfOrBlock {
    If(If),
    Block(Block),
}

impl Parse for IfOrBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![if]) {
            input.parse().map(Self::If)
        } else if lookahead.peek(Brace) {
            input.parse().map(Self::Block)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for IfOrBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::If(if_) => if_.to_tokens(tokens),
            Self::Block(block) => block.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub struct For {
    pub for_token: Token![for],
    pub pat: Pat,
    pub in_token: Token![in],
    pub expr: Expr,
    pub body: Block,
}

impl Parse for For {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            for_token: input.parse()?,
            pat: input.call(Pat::parse_multi_with_leading_vert)?,
            in_token: input.parse()?,
            expr: input.call(Expr::parse_without_eager_brace)?,
            body: input.parse()?,
        })
    }
}

impl ToTokens for For {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.for_token.to_tokens(tokens);
        self.pat.to_tokens(tokens);
        self.in_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
        self.body.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct While {
    pub while_token: Token![while],
    pub cond: Expr,
    pub body: Block,
}

impl Parse for While {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            while_token: input.parse()?,
            cond: input.call(Expr::parse_without_eager_brace)?,
            body: input.parse()?,
        })
    }
}

impl ToTokens for While {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.while_token.to_tokens(tokens);
        self.cond.to_tokens(tokens);
        self.body.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct Match {
    pub match_token: Token![match],
    pub expr: Expr,
    pub brace_token: Brace,
    pub arms: Vec<MatchArm>,
}

impl Parse for Match {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let match_token = input.parse()?;
        let expr = input.call(Expr::parse_without_eager_brace)?;

        let content;
        let brace_token = braced!(content in input);

        let mut arms = Vec::new();
        while !content.is_empty() {
            arms.push(content.parse()?);
        }

        Ok(Self {
            match_token,
            expr,
            brace_token,
            arms,
        })
    }
}

impl ToTokens for Match {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.match_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            for arm in &self.arms {
                arm.to_tokens(tokens);
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pat: Pat,
    pub guard: Option<(Token![if], Expr)>,
    pub fat_arrow_token: Token![=>],
    pub body: Markup,
    pub comma_token: Option<Token![,]>,
}

impl Parse for MatchArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pat: Pat::parse_multi_with_leading_vert(input)?,
            guard: {
                if input.peek(Token![if]) {
                    Some((input.parse()?, input.parse()?))
                } else {
                    None
                }
            },
            fat_arrow_token: input.parse()?,
            body: input.parse()?,
            comma_token: if input.peek(Token![,]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}

impl ToTokens for MatchArm {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pat.to_tokens(tokens);
        if let Some((if_token, guard)) = &self.guard {
            if_token.to_tokens(tokens);
            guard.to_tokens(tokens);
        }
        self.fat_arrow_token.to_tokens(tokens);
        self.body.to_tokens(tokens);
        if let Some(comma_token) = &self.comma_token {
            comma_token.to_tokens(tokens);
        }
    }
}
