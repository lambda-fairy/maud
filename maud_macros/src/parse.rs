use proc_macro::{
    Delimiter,
    Literal,
    Spacing,
    Span,
    TokenStream,
    TokenTree,
};
use std::collections::HashMap;
use std::mem;

use literalext::LiteralExt;

use ast;
use ParseResult;

pub fn parse(input: TokenStream) -> ParseResult<Vec<ast::Markup>> {
    Parser::new(input).markups()
}

#[derive(Clone)]
struct Parser {
    /// Indicates whether we're inside an attribute node.
    in_attr: bool,
    input: <TokenStream as IntoIterator>::IntoIter,
}

impl Iterator for Parser {
    type Item = TokenTree;

    fn next(&mut self) -> Option<TokenTree> {
        self.input.next()
    }
}

impl Parser {
    fn new(input: TokenStream) -> Parser {
        Parser {
            in_attr: false,
            input: input.into_iter(),
        }
    }

    fn with_input(&self, input: TokenStream) -> Parser {
        Parser {
            in_attr: self.in_attr,
            input: input.into_iter(),
        }
    }

    /// Returns the next token in the stream without consuming it.
    fn peek(&mut self) -> Option<TokenTree> {
        self.clone().next()
    }

    /// Returns the next two tokens in the stream without consuming them.
    fn peek2(&mut self) -> Option<(TokenTree, Option<TokenTree>)> {
        let mut clone = self.clone();
        clone.next().map(|first| (first, clone.next()))
    }

    /// Advances the cursor by one step.
    fn advance(&mut self) {
        self.next();
    }

    /// Advances the cursor by two steps.
    fn advance2(&mut self) {
        self.next();
        self.next();
    }

    /// Overwrites the current parser state with the given parameter.
    fn commit(&mut self, attempt: Parser) {
        *self = attempt;
    }

    /// Parses and renders multiple blocks of markup.
    fn markups(&mut self) -> ParseResult<Vec<ast::Markup>> {
        let mut result = Vec::new();
        loop {
            match self.peek2() {
                None => break,
                Some((TokenTree::Punct(ref punct), _)) if punct.as_char() == ';' => self.advance(),
                Some((
                    TokenTree::Punct(ref punct),
                    Some(TokenTree::Ident(ref ident)),
                )) if punct.as_char() == '@' && ident.to_string() == "let" => {
                    self.advance2();
                    let keyword = TokenTree::Ident(ident.clone());
                    result.push(self.let_expr(punct.span(), keyword)?);
                },
                _ => result.push(self.markup()?),
            }
        }
        Ok(result)
    }

    /// Parses and renders a single block of markup.
    fn markup(&mut self) -> ParseResult<ast::Markup> {
        let token = match self.peek() {
            Some(token) => token,
            None => {
                Span::call_site().error("unexpected end of input").emit();
                return Err(());
            },
        };
        let markup = match token {
            // Literal
            TokenTree::Literal(lit) => {
                self.advance();
                self.literal(&lit)?
            },
            // Special form
            TokenTree::Punct(ref punct) if punct.as_char() == '@' => {
                self.advance();
                let at_span = punct.span();
                match self.next() {
                    Some(TokenTree::Ident(ident)) => {
                        let keyword = TokenTree::Ident(ident.clone());
                        match ident.to_string().as_str() {
                            "if" => {
                                let mut segments = Vec::new();
                                self.if_expr(at_span, vec![keyword], &mut segments)?;
                                ast::Markup::Special { segments }
                            },
                            "while" => self.while_expr(at_span, keyword)?,
                            "for" => self.for_expr(at_span, keyword)?,
                            "match" => self.match_expr(at_span, keyword)?,
                            "let" => {
                                let ident_span = ident.span();
                                let span = at_span.join(ident_span).unwrap_or(ident_span);
                                span.error("`@let` only works inside a block").emit();
                                self.let_expr(at_span, keyword)?
                            },
                            other => {
                                let ident_span = ident.span();
                                let span = at_span.join(ident_span).unwrap_or(ident_span);
                                span.error(format!("unknown keyword `@{}`", other)).emit();
                                return Err(());
                            }
                        }
                    },
                    _ => {
                        at_span.error("expected keyword after `@`").emit();
                        return Err(());
                    },
                }
            },
            // Element
            TokenTree::Ident(_) => {
                let name = self.namespaced_name()?;
                self.element(name)?
            },
            // Splice
            TokenTree::Group(ref group) if group.delimiter() == Delimiter::Parenthesis => {
                self.advance();
                ast::Markup::Splice { expr: group.stream(), outer_span: group.span() }
            }
            // Block
            TokenTree::Group(ref group) if group.delimiter() == Delimiter::Brace => {
                self.advance();
                ast::Markup::Block(self.block(group.stream(), group.span())?)
            },
            // ???
            token => {
                token.span().error("invalid syntax").emit();
                return Err(());
            },
        };
        Ok(markup)
    }

    /// Parses and renders a literal string.
    fn literal(&mut self, lit: &Literal) -> ParseResult<ast::Markup> {
        let content = lit.parse_string().unwrap_or_else(|| {
            lit.span().error("expected string").emit();
            String::new()  // Insert a dummy value
        });
        Ok(ast::Markup::Literal {
            content,
            span: lit.span(),
        })
    }

    /// Parses an `@if` expression.
    ///
    /// The leading `@if` should already be consumed.
    fn if_expr(
        &mut self,
        at_span: Span,
        prefix: Vec<TokenTree>,
        segments: &mut Vec<ast::Special>,
    ) -> ParseResult<()> {
        let mut head = prefix;
        let body = loop {
            match self.next() {
                Some(TokenTree::Group(ref block)) if block.delimiter() == Delimiter::Brace => {
                    break self.block(block.stream(), block.span())?;
                },
                Some(token) => head.push(token),
                None => {
                    let head_span = ast::span_tokens(head);
                    let span = at_span.join(head_span).unwrap_or(head_span);
                    span.error("expected body for this `@if`").emit();
                    return Err(());
                },
            }
        };
        segments.push(ast::Special {
            at_span,
            head: head.into_iter().collect(),
            body,
        });
        self.else_if_expr(segments)
    }

    /// Parses an optional `@else if` or `@else`.
    ///
    /// The leading `@else if` or `@else` should *not* already be consumed.
    fn else_if_expr(&mut self, segments: &mut Vec<ast::Special>) -> ParseResult<()> {
        match self.peek2() {
            Some((
                TokenTree::Punct(ref punct),
                Some(TokenTree::Ident(ref else_keyword)),
            )) if punct.as_char() == '@' && else_keyword.to_string() == "else" => {
                self.advance2();
                let at_span = punct.span();
                let else_keyword = TokenTree::Ident(else_keyword.clone());
                match self.peek() {
                    // `@else if`
                    Some(TokenTree::Ident(ref if_keyword)) if if_keyword.to_string() == "if" => {
                        self.advance();
                        let if_keyword = TokenTree::Ident(if_keyword.clone());
                        self.if_expr(at_span, vec![else_keyword, if_keyword], segments)
                    },
                    // Just an `@else`
                    _ => {
                        match self.next() {
                            Some(TokenTree::Group(ref group)) if group.delimiter() == Delimiter::Brace => {
                                let body = self.block(group.stream(), group.span())?;
                                segments.push(ast::Special {
                                    at_span,
                                    head: vec![else_keyword].into_iter().collect(),
                                    body,
                                });
                                Ok(())
                            },
                            _ => {
                                let else_span = else_keyword.span();
                                let span = at_span.join(else_span).unwrap_or(else_span);
                                span.error("expected body for this `@else`").emit();
                                Err(())
                            },
                        }
                    },
                }
            },
            // We didn't find an `@else`; stop
            _ => Ok(()),
        }
    }

    /// Parses and renders an `@while` expression.
    ///
    /// The leading `@while` should already be consumed.
    fn while_expr(&mut self, at_span: Span, keyword: TokenTree) -> ParseResult<ast::Markup> {
        let keyword_span = keyword.span();
        let mut head = vec![keyword];
        let body = loop {
            match self.next() {
                Some(TokenTree::Group(ref block)) if block.delimiter() == Delimiter::Brace => {
                    break self.block(block.stream(), block.span())?;
                },
                Some(token) => head.push(token),
                None => {
                    let span = at_span.join(keyword_span).unwrap_or(keyword_span);
                    span.error("expected body for this `@while`").emit();
                    return Err(());
                },
            }
        };
        Ok(ast::Markup::Special {
            segments: vec![ast::Special { at_span, head: head.into_iter().collect(), body }],
        })
    }

    /// Parses a `@for` expression.
    ///
    /// The leading `@for` should already be consumed.
    fn for_expr(&mut self, at_span: Span, keyword: TokenTree) -> ParseResult<ast::Markup> {
        let keyword_span = keyword.span();
        let mut head = vec![keyword];
        loop {
            match self.next() {
                Some(TokenTree::Ident(ref in_keyword)) if in_keyword.to_string() == "in" => {
                    head.push(TokenTree::Ident(in_keyword.clone()));
                    break;
                },
                Some(token) => head.push(token),
                None => {
                    let span = at_span.join(keyword_span).unwrap_or(keyword_span);
                    span.error("missing `in` in `@for` loop").emit();
                    return Err(());
                },
            }
        }
        let body = loop {
            match self.next() {
                Some(TokenTree::Group(ref block)) if block.delimiter() == Delimiter::Brace => {
                    break self.block(block.stream(), block.span())?;
                },
                Some(token) => head.push(token),
                None => {
                    let span = at_span.join(keyword_span).unwrap_or(keyword_span);
                    span.error("expected body for this `@for`").emit();
                    return Err(());
                },
            }
        };
        Ok(ast::Markup::Special {
            segments: vec![ast::Special { at_span, head: head.into_iter().collect(), body }],
        })
    }

    /// Parses a `@match` expression.
    ///
    /// The leading `@match` should already be consumed.
    fn match_expr(&mut self, at_span: Span, keyword: TokenTree) -> ParseResult<ast::Markup> {
        let keyword_span = keyword.span();
        let mut head = vec![keyword];
        let (arms, arms_span) = loop {
            match self.next() {
                Some(TokenTree::Group(ref body)) if body.delimiter() == Delimiter::Brace => {
                    let span = body.span();
                    break (self.with_input(body.stream()).match_arms()?, span);
                },
                Some(token) => head.push(token),
                None => {
                    let span = at_span.join(keyword_span).unwrap_or(keyword_span);
                    span.error("expected body for this `@match`").emit();
                    return Err(());
                },
            }
        };
        Ok(ast::Markup::Match { at_span, head: head.into_iter().collect(), arms, arms_span })
    }

    fn match_arms(&mut self) -> ParseResult<Vec<ast::MatchArm>> {
        let mut arms = Vec::new();
        while let Some(arm) = self.match_arm()? {
            arms.push(arm);
        }
        Ok(arms)
    }

    fn match_arm(&mut self) -> ParseResult<Option<ast::MatchArm>> {
        let mut head = Vec::new();
        loop {
            match self.peek2() {
                Some((TokenTree::Punct(ref eq), Some(TokenTree::Punct(ref gt))))
                if eq.as_char() == '=' && gt.as_char() == '>' && eq.spacing() == Spacing::Joint => {
                    self.advance2();
                    head.push(TokenTree::Punct(eq.clone()));
                    head.push(TokenTree::Punct(gt.clone()));
                    break;
                },
                Some((token, _)) => {
                    self.advance();
                    head.push(token);
                },
                None => {
                    if head.is_empty() {
                        return Ok(None);
                    } else {
                        let head_span = ast::span_tokens(head);
                        head_span.error("unexpected end of @match pattern").emit();
                        return Err(());
                    }
                },
            }
        }
        let body = match self.next() {
            // $pat => { $stmts }
            Some(TokenTree::Group(ref body)) if body.delimiter() == Delimiter::Brace => {
                let body = self.block(body.stream(), body.span())?;
                // Trailing commas are optional if the match arm is a braced block
                if let Some(TokenTree::Punct(ref punct)) = self.peek() {
                    if punct.as_char() == ',' {
                        self.advance();
                    }
                }
                body
            },
            // $pat => $expr
            Some(first_token) => {
                let mut span = first_token.span();
                let mut body = vec![first_token];
                loop {
                    match self.next() {
                        Some(TokenTree::Punct(ref punct)) if punct.as_char() == ',' => break,
                        Some(token) => {
                            if let Some(bigger_span) = span.join(token.span()) {
                                span = bigger_span;
                            }
                            body.push(token);
                        },
                        None => break,
                    }
                }
                self.block(body.into_iter().collect(), span)?
            },
            None => {
                let span = ast::span_tokens(head);
                span.error("unexpected end of @match arm").emit();
                return Err(());
            },
        };
        Ok(Some(ast::MatchArm { head: head.into_iter().collect(), body }))
    }

    /// Parses a `@let` expression.
    ///
    /// The leading `@let` should already be consumed.
    fn let_expr(&mut self, at_span: Span, keyword: TokenTree) -> ParseResult<ast::Markup> {
        let mut tokens = vec![keyword];
        loop {
            match self.next() {
                Some(token) => {
                    match token {
                        TokenTree::Punct(ref punct) if punct.as_char() == '=' => {
                            tokens.push(token.clone());
                            break;
                        },
                        _ => tokens.push(token),
                    }
                },
                None => {
                    let tokens_span = ast::span_tokens(tokens);
                    let span = at_span.join(tokens_span).unwrap_or(tokens_span);
                    span.error("unexpected end of `@let` expression").emit();
                    return Err(());
                }
            }
        }
        loop {
            match self.next() {
                Some(token) => {
                    match token {
                        TokenTree::Punct(ref punct) if punct.as_char() == ';' => {
                            tokens.push(token.clone());
                            break;
                        },
                        _ => tokens.push(token),
                    }
                },
                None => {
                    let tokens_span = ast::span_tokens(tokens);
                    let span = at_span.join(tokens_span).unwrap_or(tokens_span);
                    span.error("unexpected end of `@let` expression (are you missing a semicolon?)")
                        .emit();
                    return Err(());
                },
            }
        }
        Ok(ast::Markup::Let { at_span, tokens: tokens.into_iter().collect() })
    }

    /// Parses an element node.
    ///
    /// The element name should already be consumed.
    fn element(&mut self, name: TokenStream) -> ParseResult<ast::Markup> {
        if self.in_attr {
            let span = ast::span_tokens(name);
            span.error("unexpected element, you silly bumpkin").emit();
            return Err(());
        }
        let attrs = self.attrs()?;
        let body = match self.peek() {
            Some(TokenTree::Punct(ref punct))
            if punct.as_char() == ';' || punct.as_char() == '/' => {
                // Void element
                self.advance();
                ast::ElementBody::Void { semi_span: punct.span() }
            },
            _ => {
                match self.markup()? {
                    ast::Markup::Block(block) => ast::ElementBody::Block { block },
                    markup => {
                        let markup_span = markup.span();
                        markup_span
                            .error("element body must be wrapped in braces")
                            .help("see https://github.com/lfairy/maud/pull/137 for details")
                            .emit();
                        ast::ElementBody::Block {
                            block: ast::Block {
                                markups: vec![markup],
                                outer_span: markup_span,
                            },
                        }
                    },
                }
            },
        };
        Ok(ast::Markup::Element { name, attrs, body })
    }

    /// Parses the attributes of an element.
    fn attrs(&mut self) -> ParseResult<ast::Attrs> {
        let mut attrs = Vec::new();
        loop {
            let mut attempt = self.clone();
            let maybe_name = attempt.try_namespaced_name();
            let token_after = attempt.next();
            match (maybe_name, token_after) {
                // Non-empty attribute
                (Some(ref name), Some(TokenTree::Punct(ref punct))) if punct.as_char() == '=' => {
                    self.commit(attempt);
                    let value;
                    {
                        // Parse a value under an attribute context
                        let in_attr = mem::replace(&mut self.in_attr, true);
                        value = self.markup()?;
                        self.in_attr = in_attr;
                    }
                    attrs.push(ast::Attr::Attribute {
                        attribute: ast::Attribute {
                            name: name.clone(),
                            attr_type: ast::AttrType::Normal { value },
                        },
                    });
                },
                // Empty attribute
                (Some(ref name), Some(TokenTree::Punct(ref punct))) if punct.as_char() == '?' => {
                    self.commit(attempt);
                    let toggler = self.attr_toggler();
                    attrs.push(ast::Attr::Attribute {
                        attribute: ast::Attribute {
                            name: name.clone(),
                            attr_type: ast::AttrType::Empty { toggler },
                        },
                    });
                },
                // Class shorthand
                (None, Some(TokenTree::Punct(ref punct))) if punct.as_char() == '.' => {
                    self.commit(attempt);
                    let name = self.name()?;
                    let toggler = self.attr_toggler();
                    attrs.push(ast::Attr::Class { name, toggler });
                },
                // ID shorthand
                (None, Some(TokenTree::Punct(ref punct))) if punct.as_char() == '#' => {
                    self.commit(attempt);
                    let name = self.name()?;
                    attrs.push(ast::Attr::Id { name });
                },
                // If it's not a valid attribute, backtrack and bail out
                _ => break,
            }
        }

        let mut attr_map: HashMap<String, Vec<Span>> = HashMap::new();
        let mut has_class = false;
        for attr in &attrs {
            let name = match attr {
                ast::Attr::Class { .. } => {
                    if has_class {
                        // Only check the first class to avoid spurious duplicates
                        continue;
                    }
                    has_class = true;
                    "class".to_string()
                },
                ast::Attr::Id { .. } => "id".to_string(),
                ast::Attr::Attribute { attribute } => {
                    attribute.name.clone().into_iter().map(|token| token.to_string()).collect()
                },
            };
            let entry = attr_map.entry(name).or_default();
            entry.push(attr.span());
        }

        for (name, spans) in attr_map {
            if spans.len() > 1 {
                let mut spans = spans.into_iter();
                let first_span = spans.next().expect("spans should be non-empty");
                spans
                    .fold(
                        first_span.error(format!("duplicate attribute `{}`", name)),
                        |acc, span| acc.span_note(span, format!("`{}` is duplicated here", name)),
                    )
                    .emit();
            }
        }

        Ok(attrs)
    }

    /// Parses the `[cond]` syntax after an empty attribute or class shorthand.
    fn attr_toggler(&mut self) -> Option<ast::Toggler> {
        match self.peek() {
            Some(TokenTree::Group(ref group)) if group.delimiter() == Delimiter::Bracket => {
                self.advance();
                Some(ast::Toggler {
                    cond: group.stream(),
                    cond_span: group.span(),
                })
            },
            _ => None,
        }
    }

    /// Parses an identifier, without dealing with namespaces.
    fn name(&mut self) -> ParseResult<TokenStream> {
        self.try_name().ok_or_else(|| {
            Span::call_site().error("expected identifier").emit();
        })
    }

    fn try_name(&mut self) -> Option<TokenStream> {
        let mut result = Vec::new();
        if let Some(token @ TokenTree::Ident(_)) = self.peek() {
            self.advance();
            result.push(token);
        } else {
            return None;
        }
        let mut expect_ident = false;
        loop {
            expect_ident = match self.peek() {
                Some(TokenTree::Punct(ref punct)) if punct.as_char() == '-' => {
                    self.advance();
                    result.push(TokenTree::Punct(punct.clone()));
                    true
                },
                Some(TokenTree::Ident(ref ident)) if expect_ident => {
                    self.advance();
                    result.push(TokenTree::Ident(ident.clone()));
                    false
                },
                _ => break,
            };
        }
        Some(result.into_iter().collect())
    }

    /// Parses a HTML element or attribute name, along with a namespace
    /// if necessary.
    fn namespaced_name(&mut self) -> ParseResult<TokenStream> {
        self.try_namespaced_name().ok_or_else(|| {
            Span::call_site().error("expected identifier").emit();
        })
    }

    fn try_namespaced_name(&mut self) -> Option<TokenStream> {
        let mut result = vec![self.try_name()?];
        if let Some(TokenTree::Punct(ref punct)) = self.peek() {
            if punct.as_char() == ':' {
                self.advance();
                result.push(TokenStream::from(TokenTree::Punct(punct.clone())));
                result.push(self.try_name()?);
            }
        }
        Some(result.into_iter().collect())
    }

    /// Parses the given token stream as a Maud expression.
    fn block(&mut self, body: TokenStream, outer_span: Span) -> ParseResult<ast::Block> {
        let markups = self.with_input(body).markups()?;
        Ok(ast::Block { markups, outer_span })
    }
}
