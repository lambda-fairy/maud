use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, token::Brace, Expr, Local};

use crate::{ast::*, escape};

pub fn generate(markups: Markups<Element>, output_ident: Ident, as_struct: bool) -> TokenStream {
    let mut build = Builder::new(output_ident.clone());
    Generator::new(output_ident, as_struct).markups(markups, &mut build);
    build.finish()
}

struct Generator {
    output_ident: Ident,
    as_struct: bool,
}

impl Generator {
    fn new(output_ident: Ident, as_struct: bool) -> Generator {
        Generator {
            output_ident,
            as_struct,
        }
    }

    fn builder(&self) -> Builder {
        Builder::new(self.output_ident.clone())
    }

    fn markups<E: Into<Element>>(&self, markups: Markups<E>, build: &mut Builder) {
        for markup in markups.markups {
            self.markup(markup, build);
        }
    }

    fn markup<E: Into<Element>>(&self, markup: Markup<E>, build: &mut Builder) {
        match markup {
            Markup::Block(block) => {
                if block.markups.markups.iter().any(|markup| {
                    matches!(
                        *markup,
                        Markup::ControlFlow(ControlFlow {
                            kind: ControlFlowKind::Let(_),
                            ..
                        })
                    )
                }) {
                    self.block(block, build);
                } else {
                    self.markups(block.markups, build);
                }
            }
            Markup::Lit(lit) => build.push_escaped(&lit.to_string()),
            Markup::Splice { expr, .. } => self.splice(expr, build),
            Markup::Element(element) => self.element(element.into(), build),
            Markup::ControlFlow(control_flow) => self.control_flow(control_flow, build),
            Markup::Semi(_) => {}
        }
    }

    fn block<E: Into<Element>>(&self, block: Block<E>, build: &mut Builder) {
        let markups = {
            let mut build = self.builder();
            self.markups(block.markups, &mut build);
            build.finish()
        };

        build.push_tokens(quote!({ #markups }));
    }

    fn splice(&self, expr: Expr, build: &mut Builder) {
        let output_ident = &self.output_ident;
        if self.as_struct {
            build.push_tokens(quote!(maud::macro_private::render_to!(&(#expr), #output_ident);));
        } else {
            build.push_tokens(
                quote!(maud::macro_private::render_to!(&(#expr), &mut #output_ident);),
            );
        }
    }

    fn element(&self, element: Element, build: &mut Builder) {
        let element_name = element.name.clone().unwrap_or_else(|| parse_quote!(div));
        build.push_str("<");
        self.name(element_name.clone(), build);
        self.attrs(element.attrs, build);
        build.push_str(">");
        if let ElementBody::Block(block) = element.body {
            self.markups(block.markups, build);
            build.push_str("</");
            self.name(element_name, build);
            build.push_str(">");
        }
    }

    fn name(&self, name: HtmlName, build: &mut Builder) {
        build.push_escaped(&name.to_string());
    }

    fn name_or_markup(&self, name: HtmlNameOrMarkup, build: &mut Builder) {
        match name {
            HtmlNameOrMarkup::HtmlName(name) => self.name(name, build),
            HtmlNameOrMarkup::Markup(markup) => self.markup(markup, build),
        }
    }

    fn attr(&self, name: HtmlName, value: AttributeType, build: &mut Builder) {
        match value {
            AttributeType::Normal { value, .. } => {
                build.push_str(" ");
                self.name(name, build);
                build.push_str("=\"");
                self.markup(value, build);
                build.push_str("\"");
            }
            AttributeType::Optional {
                toggler: Toggler { cond, .. },
                ..
            } => {
                let inner_value: Expr = parse_quote!(inner_value);

                let body = {
                    let mut build = self.builder();
                    build.push_str(" ");
                    self.name(name, &mut build);
                    build.push_str("=\"");
                    self.splice(inner_value.clone(), &mut build);
                    build.push_str("\"");
                    build.finish()
                };
                build.push_tokens(quote!(if let Some(#inner_value) = (#cond) { #body }));
            }
            AttributeType::Empty(None) => {
                build.push_str(" ");
                self.name(name, build);
            }
            AttributeType::Empty(Some(Toggler { cond, .. })) => {
                let body = {
                    let mut build = self.builder();
                    build.push_str(" ");
                    self.name(name, &mut build);
                    build.finish()
                };
                build.push_tokens(quote!(if (#cond) { #body }));
            }
        }
    }

    fn attrs(&self, attrs: Vec<Attribute>, build: &mut Builder) {
        let (classes, id, named_attrs) = split_attrs(attrs);

        if !classes.is_empty() {
            let mut toggle_class_exprs = vec![];

            build.push_str(" ");
            self.name(parse_quote!(class), build);
            build.push_str("=\"");
            for (i, (name, toggler)) in classes.into_iter().enumerate() {
                if let Some(toggler) = toggler {
                    toggle_class_exprs.push((i > 0, name, toggler));
                } else {
                    if i > 0 {
                        build.push_str(" ");
                    }
                    self.name_or_markup(name, build);
                }
            }

            for (not_first, name, toggler) in toggle_class_exprs {
                let body = {
                    let mut build = self.builder();
                    if not_first {
                        build.push_str(" ");
                    }
                    self.name_or_markup(name, &mut build);
                    build.finish()
                };
                build.push_tokens(quote!(if (#toggler) { #body }));
            }

            build.push_str("\"");
        }

        if let Some(id) = id {
            build.push_str(" ");
            self.name(parse_quote!(id), build);
            build.push_str("=\"");
            self.name_or_markup(id, build);
            build.push_str("\"");
        }

        for (name, attr_type) in named_attrs {
            self.attr(name, attr_type, build);
        }
    }

    fn control_flow<E: Into<Element>>(&self, control_flow: ControlFlow<E>, build: &mut Builder) {
        match control_flow.kind {
            ControlFlowKind::If(if_) => self.control_flow_if(if_, build),
            ControlFlowKind::Let(let_) => self.control_flow_let(let_, build),
            ControlFlowKind::For(for_) => self.control_flow_for(for_, build),
            ControlFlowKind::While(while_) => self.control_flow_while(while_, build),
            ControlFlowKind::Match(match_) => self.control_flow_match(match_, build),
        }
    }

    fn control_flow_if<E: Into<Element>>(
        &self,
        IfExpr {
            if_token,
            cond,
            then_branch,
            else_branch,
        }: IfExpr<E>,
        build: &mut Builder,
    ) {
        build.push_tokens(quote!(#if_token #cond));
        self.block(then_branch, build);

        if let Some((_, else_token, if_or_block)) = else_branch {
            build.push_tokens(quote!(#else_token));
            self.control_flow_if_or_block(*if_or_block, build);
        }
    }

    fn control_flow_if_or_block<E: Into<Element>>(
        &self,
        if_or_block: IfOrBlock<E>,
        build: &mut Builder,
    ) {
        match if_or_block {
            IfOrBlock::If(if_) => self.control_flow_if(if_, build),
            IfOrBlock::Block(block) => self.block(block, build),
        }
    }

    fn control_flow_let(&self, let_: Local, build: &mut Builder) {
        build.push_tokens(let_.to_token_stream());
    }

    fn control_flow_for<E: Into<Element>>(
        &self,
        ForExpr {
            for_token,
            pat,
            in_token,
            expr,
            body,
        }: ForExpr<E>,
        build: &mut Builder,
    ) {
        build.push_tokens(quote!(#for_token #pat #in_token (#expr)));
        self.block(body, build);
    }

    fn control_flow_while<E: Into<Element>>(
        &self,
        WhileExpr {
            while_token,
            cond,
            body,
        }: WhileExpr<E>,
        build: &mut Builder,
    ) {
        build.push_tokens(quote!(#while_token #cond));
        self.block(body, build);
    }

    fn control_flow_match<E: Into<Element>>(
        &self,
        MatchExpr {
            match_token,
            expr,
            brace_token,
            arms,
        }: MatchExpr<E>,
        build: &mut Builder,
    ) {
        let arms = {
            let mut build = self.builder();
            for MatchArm {
                pat,
                guard,
                fat_arrow_token,
                body,
                comma_token,
            } in arms
            {
                build.push_tokens(quote!(#pat));
                if let Some((if_token, cond)) = guard {
                    build.push_tokens(quote!(#if_token #cond));
                }
                build.push_tokens(quote!(#fat_arrow_token));
                self.block(
                    Block {
                        brace_token: Brace(Span::call_site()),
                        markups: Markups {
                            markups: vec![body],
                        },
                    },
                    &mut build,
                );
                build.push_tokens(quote!(#comma_token));
            }
            build.finish()
        };

        let mut arm_block = TokenStream::new();

        brace_token.surround(&mut arm_block, |tokens| {
            arms.to_tokens(tokens);
        });

        build.push_tokens(quote!(#match_token #expr #arm_block));
    }
}

////////////////////////////////////////////////////////

#[allow(clippy::type_complexity)]
fn split_attrs(
    attrs: Vec<Attribute>,
) -> (
    Vec<(HtmlNameOrMarkup, Option<Expr>)>,
    Option<HtmlNameOrMarkup>,
    Vec<(HtmlName, AttributeType)>,
) {
    let mut classes = vec![];
    let mut id = None;
    let mut named_attrs = vec![];

    for attr in attrs {
        match attr {
            Attribute::Class { name, toggler, .. } => {
                classes.push((name, toggler.map(|toggler| toggler.cond)))
            }
            Attribute::Id { name, .. } => id = Some(name),
            Attribute::Named { name, attr_type } => named_attrs.push((name, attr_type)),
        }
    }

    (classes, id, named_attrs)
}

////////////////////////////////////////////////////////

struct Builder {
    output_ident: Ident,
    tokens: TokenStream,
    tail: String,
}

impl Builder {
    fn new(output_ident: Ident) -> Builder {
        Builder {
            output_ident,
            tokens: TokenStream::new(),
            tail: String::new(),
        }
    }

    fn push_str(&mut self, string: &'static str) {
        self.tail.push_str(string);
    }

    fn push_escaped(&mut self, string: &str) {
        escape::escape_to_string(string, &mut self.tail);
    }

    fn push_tokens(&mut self, tokens: TokenStream) {
        self.cut();
        self.tokens.extend(tokens);
    }

    fn cut(&mut self) {
        if self.tail.is_empty() {
            return;
        }
        let push_str_expr = {
            let output_ident = self.output_ident.clone();
            let tail = &self.tail;
            quote!(#output_ident.push_str(#tail);)
        };
        self.tail.clear();
        self.tokens.extend(push_str_expr);
    }

    fn finish(mut self) -> TokenStream {
        self.cut();
        self.tokens
    }
}
