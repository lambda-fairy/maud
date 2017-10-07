use proc_macro::{
    Delimiter,
    Literal,
    Spacing,
    Span,
    TokenNode,
    TokenStream,
    TokenTree,
    TokenTreeIter,
};
use std::iter;
use std::mem;

use literalext::LiteralExt;

use super::build::Builder;
use super::ParseResult;

pub fn parse(input: TokenStream, output_ident: TokenTree) -> ParseResult<TokenStream> {
    let mut parser = Parser::new(input, output_ident);
    let mut builder = parser.builder();
    parser.markups(&mut builder)?;
    Ok(builder.build())
}

#[derive(Clone)]
struct Parser {
    output_ident: TokenTree,
    /// Indicates whether we're inside an attribute node.
    in_attr: bool,
    input: TokenTreeIter,
}

impl Iterator for Parser {
    type Item = TokenTree;

    fn next(&mut self) -> Option<TokenTree> {
        self.input.next()
    }
}

impl Parser {
    fn new(input: TokenStream, output_ident: TokenTree) -> Parser {
        Parser {
            output_ident,
            in_attr: false,
            input: input.into_iter(),
        }
    }

    fn with_input(&self, input: TokenStream) -> Parser {
        Parser {
            output_ident: self.output_ident.clone(),
            in_attr: self.in_attr,
            input: input.into_iter(),
        }
    }

    fn builder(&self) -> Builder {
        Builder::new(self.output_ident.clone())
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

    /// Returns an `Err` with the given message.
    fn error<T, E: Into<String>>(&self, message: E) -> ParseResult<T> {
        Err(message.into())
    }

    /// Parses and renders multiple blocks of markup.
    fn markups(&mut self, builder: &mut Builder) -> ParseResult<()> {
        loop {
            match self.peek2() {
                None => return Ok(()),
                Some((TokenTree { kind: TokenNode::Op(';', _), .. }, _)) => self.advance(),
                Some((
                    TokenTree { kind: TokenNode::Op('@', _), .. },
                    Some(TokenTree { kind: TokenNode::Term(term), span }),
                )) if term.as_str() == "let" => {
                    // When emitting a `@let`, wrap the rest of the block in a
                    // new block to avoid scoping issues
                    let keyword = TokenTree { kind: TokenNode::Term(term), span };
                    self.advance2();
                    builder.push({
                        let mut builder = self.builder();
                        builder.push(keyword);
                        self.let_expr(&mut builder)?;
                        self.markups(&mut builder)?;
                        TokenTree {
                            kind: TokenNode::Group(Delimiter::Brace, builder.build()),
                            span,
                        }
                    });
                },
                _ => self.markup(builder)?,
            }
        }
    }

    /// Parses and renders a single block of markup.
    fn markup(&mut self, builder: &mut Builder) -> ParseResult<()> {
        let token = match self.peek() {
            Some(token) => token,
            None => return self.error("unexpected end of input"),
        };
        match token {
            // Literal
            TokenTree { kind: TokenNode::Literal(lit), .. } => {
                self.advance();
                self.literal(lit, builder)?;
            },
            // Special form
            TokenTree { kind: TokenNode::Op('@', _), .. } => {
                self.advance();
                match self.next() {
                    Some(TokenTree { kind: TokenNode::Term(term), span }) => {
                        let keyword = TokenTree { kind: TokenNode::Term(term), span };
                        builder.push(keyword);
                        match term.as_str() {
                            "if" => self.if_expr(builder)?,
                            "while" => self.while_expr(builder)?,
                            "for" => self.for_expr(builder)?,
                            "match" => self.match_expr(builder)?,
                            "let" => return self.error(format!("@let only works inside a block")),
                            other => return self.error(format!("unknown keyword `@{}`", other)),
                        }
                    },
                    _ => return self.error("expected keyword after `@`"),
                }
            }
            // Element
            TokenTree { kind: TokenNode::Term(_), .. } => {
                let name = self.namespaced_name()?;
                self.element(&name, builder)?;
            },
            // Splice
            TokenTree { kind: TokenNode::Group(Delimiter::Parenthesis, expr), .. } => {
                self.advance();
                builder.splice(expr);
            }
            // Block
            TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), .. } => {
                self.advance();
                self.with_input(block).markups(builder)?;
            },
            // ???
            _ => return self.error("invalid syntax"),
        }
        Ok(())
    }

    /// Parses and renders a literal string.
    fn literal(&mut self, lit: Literal, builder: &mut Builder) -> ParseResult<()> {
        if let Some(s) = lit.parse_string() {
            builder.string(&s);
            Ok(())
        } else {
            self.error("expected string")
        }
    }

    /// Parses and renders an `@if` expression.
    ///
    /// The leading `@if` should already be consumed.
    fn if_expr(&mut self, builder: &mut Builder) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span }) => {
                    let block = self.block(block, span)?;
                    builder.push(block);
                    break;
                },
                Some(token) => builder.push(token),
                None => return self.error("unexpected end of @if expression"),
            }
        }
        self.else_if_expr(builder)
    }

    /// Parses and renders an optional `@else if` or `@else`.
    ///
    /// The leading `@else if` or `@else` should *not* already be consumed.
    fn else_if_expr(&mut self, builder: &mut Builder) -> ParseResult<()> {
        match self.peek2() {
            // Try to match an `@else` after this
            Some((
                TokenTree { kind: TokenNode::Op('@', _), .. },
                Some(TokenTree { kind: TokenNode::Term(else_keyword), span }),
            )) if else_keyword.as_str() == "else" => {
                self.advance2();
                let else_keyword = TokenTree { kind: TokenNode::Term(else_keyword), span };
                builder.push(else_keyword);
                match self.peek() {
                    // `@else if`
                    Some(TokenTree { kind: TokenNode::Term(if_keyword), span })
                    if if_keyword.as_str() == "if" => {
                        self.advance();
                        let if_keyword = TokenTree { kind: TokenNode::Term(if_keyword), span };
                        builder.push(if_keyword);
                        self.if_expr(builder)?;
                    },
                    // Just an `@else`
                    _ => {
                        if let Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span }) = self.next() {
                            let block = self.block(block, span)?;
                            builder.push(block);
                        } else {
                            return self.error("expected body for @else");
                        }
                    },
                }
                self.else_if_expr(builder)
            },
            // We didn't find an `@else`; stop
            _ => Ok(()),
        }
    }

    /// Parses and renders an `@while` expression.
    ///
    /// The leading `@while` should already be consumed.
    fn while_expr(&mut self, builder: &mut Builder) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span }) => {
                    let block = self.block(block, span)?;
                    builder.push(block);
                    break;
                },
                Some(token) => builder.push(token),
                None => return self.error("unexpected end of @while expression"),
            }
        }
        Ok(())
    }

    /// Parses and renders a `@for` expression.
    ///
    /// The leading `@for` should already be consumed.
    fn for_expr(&mut self, builder: &mut Builder) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Term(in_keyword), span }) if in_keyword.as_str() == "in" => {
                    builder.push(TokenTree { kind: TokenNode::Term(in_keyword), span });
                    break;
                },
                Some(token) => builder.push(token),
                None => return self.error("unexpected end of @for expression"),
            }
        }
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span }) => {
                    let block = self.block(block, span)?;
                    builder.push(block);
                    break;
                },
                Some(token) => builder.push(token),
                None => return self.error("unexpected end of @for expression"),
            }
        }
        Ok(())
    }

    /// Parses and renders a `@match` expression.
    ///
    /// The leading `@match` should already be consumed.
    fn match_expr(&mut self, builder: &mut Builder) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, body), span }) => {
                    let body = self.with_input(body).match_arms()?;
                    builder.push(TokenTree {
                        kind: TokenNode::Group(Delimiter::Brace, body),
                        span,
                    });
                    break;
                },
                Some(token) => builder.push(token),
                None => return self.error("unexpected end of @match expression"),
            }
        }
        Ok(())
    }

    fn match_arms(&mut self) -> ParseResult<TokenStream> {
        let mut arms = Vec::new();
        while let Some(arm) = self.match_arm()? {
            arms.push(arm);
        }
        Ok(arms.into_iter().collect())
    }

    fn match_arm(&mut self) -> ParseResult<Option<TokenStream>> {
        let mut pat = Vec::new();
        loop {
            match self.peek2() {
                Some((
                    eq @ TokenTree { kind: TokenNode::Op('=', Spacing::Joint), .. },
                    Some(gt @ TokenTree { kind: TokenNode::Op('>', _), .. }),
                )) => {
                    self.advance2();
                    pat.push(eq);
                    pat.push(gt);
                    break;
                },
                Some((token, _)) => {
                    self.advance();
                    pat.push(token);
                },
                None =>
                    if pat.is_empty() {
                        return Ok(None);
                    } else {
                        return self.error("unexpected end of @match pattern");
                    },
            }
        }
        let body = match self.next() {
            // $pat => { $stmts }
            Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, body), span }) => {
                let body = self.block(body, span)?;
                // Trailing commas are optional if the match arm is a braced block
                if let Some(TokenTree { kind: TokenNode::Op(',', _), .. }) = self.peek() {
                    self.advance();
                }
                body
            },
            // $pat => $expr
            Some(first_token) => {
                let mut body = vec![first_token];
                loop {
                    match self.next() {
                        Some(TokenTree { kind: TokenNode::Op(',', _), .. }) => break,
                        Some(token) => {
                            body.push(token);
                        },
                        None => return self.error("unexpected end of @match arm"),
                    }
                }
                self.block(body.into_iter().collect(), Span::default())?
            },
            None => return self.error("unexpected end of @match arm"),
        };
        Ok(Some(pat.into_iter().chain(iter::once(body)).collect()))
    }

    /// Parses and renders a `@let` expression.
    ///
    /// The leading `@let` should already be consumed.
    fn let_expr(&mut self, builder: &mut Builder) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(token @ TokenTree { kind: TokenNode::Op('=', _), .. }) => {
                    builder.push(token);
                    break;
                },
                Some(token) => builder.push(token),
                None => return self.error("unexpected end of @let expression"),
            }
        }
        loop {
            match self.next() {
                Some(token @ TokenTree { kind: TokenNode::Op(';', _), .. }) => {
                    builder.push(token);
                    break;
                },
                Some(token) => builder.push(token),
                None => return self.error("unexpected end of @let expression"),
            }
        }
        Ok(())
    }

    /// Parses and renders an element node.
    ///
    /// The element name should already be consumed.
    fn element(&mut self, name: &str, builder: &mut Builder) -> ParseResult<()> {
        if self.in_attr {
            return self.error("unexpected element, you silly bumpkin");
        }
        builder.element_open_start(name);
        self.attrs(builder)?;
        builder.element_open_end();
        match self.peek() {
            Some(TokenTree { kind: TokenNode::Op(';', _), .. }) |
            Some(TokenTree { kind: TokenNode::Op('/', _), .. }) => {
                // Void element
                self.advance();
            },
            _ => {
                self.markup(builder)?;
                builder.element_close(name);
            },
        }
        Ok(())
    }

    /// Parses and renders the attributes of an element.
    fn attrs(&mut self, builder: &mut Builder) -> ParseResult<()> {
        let mut classes_static = Vec::new();
        let mut classes_toggled = Vec::new();
        let mut ids = Vec::new();
        loop {
            let mut attempt = self.clone();
            let maybe_name = attempt.namespaced_name();
            let token_after = attempt.next();
            match (maybe_name, token_after) {
                // Non-empty attribute
                (Ok(name), Some(TokenTree { kind: TokenNode::Op('=', _), .. })) => {
                    self.commit(attempt);
                    builder.attribute_start(&name);
                    {
                        // Parse a value under an attribute context
                        let in_attr = mem::replace(&mut self.in_attr, true);
                        self.markup(builder)?;
                        self.in_attr = in_attr;
                    }
                    builder.attribute_end();
                },
                // Empty attribute
                (Ok(name), Some(TokenTree { kind: TokenNode::Op('?', _), .. })) => {
                    self.commit(attempt);
                    if let Some((cond, cond_span)) = self.attr_toggler() {
                        // Toggle the attribute based on a boolean expression
                        let body = {
                            let mut builder = self.builder();
                            builder.attribute_empty(&name);
                            builder.build()
                        };
                        builder.emit_if(cond, cond_span, body);
                    } else {
                        // Write the attribute unconditionally
                        builder.attribute_empty(&name);
                    }
                },
                // Class shorthand
                (Err(_), Some(TokenTree { kind: TokenNode::Op('.', _), .. })) => {
                    self.commit(attempt);
                    let class_name = self.name()?;
                    if let Some((cond, cond_span)) = self.attr_toggler() {
                        // Toggle the class based on a boolean expression
                        classes_toggled.push((cond, cond_span, class_name));
                    } else {
                        // Emit the class unconditionally
                        classes_static.push(class_name);
                    }
                },
                // ID shorthand
                (Err(_), Some(TokenTree { kind: TokenNode::Op('#', _), .. })) => {
                    self.commit(attempt);
                    ids.push(self.name()?);
                },
                // If it's not a valid attribute, backtrack and bail out
                _ => break,
            }
        }
        if !classes_static.is_empty() || !classes_toggled.is_empty() {
            builder.attribute_start("class");
            builder.string(&classes_static.join(" "));
            for (i, (cond, cond_span, mut class_name)) in classes_toggled.into_iter().enumerate() {
                // If a class comes first in the list, then it shouldn't be
                // prefixed by a space
                if i > 0 || !classes_static.is_empty() {
                    class_name = format!(" {}", class_name);
                }
                let body = {
                    let mut builder = self.builder();
                    builder.string(&class_name);
                    builder.build()
                };
                builder.emit_if(cond, cond_span, body);
            }
            builder.attribute_end();
        }
        if !ids.is_empty() {
            builder.attribute_start("id");
            builder.string(&ids.join(" "));
            builder.attribute_end();
        }
        Ok(())
    }

    /// Parses the `[cond]` syntax after an empty attribute or class shorthand.
    fn attr_toggler(&mut self) -> Option<(TokenStream, Span)> {
        if let Some(TokenTree {
            kind: TokenNode::Group(Delimiter::Bracket, cond),
            span: delim_span,
        }) = self.peek() {
            self.advance();
            Some((cond, delim_span))
        } else {
            None
        }
    }

    /// Parses an identifier, without dealing with namespaces.
    fn name(&mut self) -> ParseResult<String> {
        let mut s = if let Some(TokenTree { kind: TokenNode::Term(term), .. }) = self.peek() {
            self.advance();
            String::from(term.as_str())
        } else {
            return self.error("expected identifier");
        };
        let mut expect_ident = false;
        loop {
            expect_ident = match self.peek() {
                Some(TokenTree { kind: TokenNode::Op('-', _), .. }) => {
                    self.advance();
                    s.push('-');
                    true
                },
                Some(TokenTree { kind: TokenNode::Term(term), .. }) if expect_ident => {
                    self.advance();
                    s.push_str(term.as_str());
                    false
                },
                _ => break,
            };
        }
        Ok(s)
    }

    /// Parses a HTML element or attribute name, along with a namespace
    /// if necessary.
    fn namespaced_name(&mut self) -> ParseResult<String> {
        let mut s = self.name()?;
        if let Some(TokenTree { kind: TokenNode::Op(':', _), .. }) = self.peek() {
            self.advance();
            s.push(':');
            s.push_str(&self.name()?);
        }
        Ok(s)
    }

    /// Parses the given token stream as a Maud expression, returning a block of
    /// Rust code.
    fn block(&mut self, body: TokenStream, span: Span) -> ParseResult<TokenTree> {
        let mut builder = self.builder();
        self.with_input(body).markups(&mut builder)?;
        Ok(TokenTree {
            kind: TokenNode::Group(Delimiter::Brace, builder.build()),
            span,
        })
    }
}
