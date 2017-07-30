use proc_macro::{
    Delimiter,
    Literal,
    Spacing,
    Span,
    Term,
    TokenNode,
    TokenStream,
    TokenTree,
    TokenTreeIter,
};
use std::iter;
use std::mem;

use literalext::LiteralExt;

use super::render::Renderer;
use super::ParseResult;

pub fn parse(input: TokenStream) -> ParseResult<TokenStream> {
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let mut render = Renderer::new();
    Parser {
        in_attr: false,
        input: input.into_iter(),
    }.markups(&mut render)?;
    Ok(render.into_expr(size_hint))
}

#[derive(Clone)]
struct Parser {
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
    fn markups(&mut self, render: &mut Renderer) -> ParseResult<()> {
        loop {
            match self.peek() {
                None => return Ok(()),
                Some(TokenTree { kind: TokenNode::Op(';', _), .. }) => self.advance(),
                _ => self.markup(render)?,
            }
        }
    }

    /// Parses and renders a single block of markup.
    fn markup(&mut self, render: &mut Renderer) -> ParseResult<()> {
        let token = match self.peek() {
            Some(token) => token,
            None => return self.error("unexpected end of input"),
        };
        match token {
            // Literal
            TokenTree { kind: TokenNode::Literal(lit), .. } => {
                self.advance();
                self.literal(lit, render)?;
            },
            // Special form
            TokenTree { kind: TokenNode::Op('@', _), .. } => {
                self.advance();
                match self.next() {
                    Some(TokenTree { kind: TokenNode::Term(term), span }) => {
                        let keyword = TokenTree { kind: TokenNode::Term(term), span };
                        render.push(keyword);
                        match term.as_str() {
                            "if" => self.if_expr(render)?,
                            "while" => self.while_expr(render)?,
                            "for" => self.for_expr(render)?,
                            "match" => self.match_expr(render)?,
                            "let" => self.let_expr(render)?,
                            other => return self.error(format!("unknown keyword `@{}`", other)),
                        }
                    },
                    _ => return self.error("expected keyword after `@`"),
                }
            }
            // Element
            TokenTree { kind: TokenNode::Term(_), .. } => {
                let name = self.namespaced_name()?;
                self.element(&name, render)?;
            },
            // Splice
            TokenTree { kind: TokenNode::Group(Delimiter::Parenthesis, expr), .. } => {
                self.advance();
                render.splice(expr);
            }
            // Block
            TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span } => {
                self.advance();
                let block = self.block(block, span, render)?;
                render.push(block);
            },
            // ???
            _ => return self.error("invalid syntax"),
        }
        Ok(())
    }

    /// Parses and renders a literal string.
    fn literal(&mut self, lit: Literal, render: &mut Renderer) -> ParseResult<()> {
        if let Some(s) = lit.parse_string() {
            render.string(&s);
            Ok(())
        } else {
            self.error("expected string")
        }
    }

    /// Parses and renders an `@if` expression.
    ///
    /// The leading `@if` should already be consumed.
    fn if_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span }) => {
                    let block = self.block(block, span, render)?;
                    render.push(block);
                    break;
                },
                Some(token) => render.push(token),
                None => return self.error("unexpected end of @if expression"),
            }
        }
        self.else_if_expr(render)
    }

    /// Parses and renders an optional `@else if` or `@else`.
    ///
    /// The leading `@else if` or `@else` should *not* already be consumed.
    fn else_if_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        match self.peek2() {
            // Try to match an `@else` after this
            Some((
                TokenTree { kind: TokenNode::Op('@', _), .. },
                Some(TokenTree { kind: TokenNode::Term(else_keyword), span }),
            )) if else_keyword.as_str() == "else" => {
                self.advance2();
                let else_keyword = TokenTree { kind: TokenNode::Term(else_keyword), span };
                render.push(else_keyword);
                match self.peek() {
                    // `@else if`
                    Some(TokenTree { kind: TokenNode::Term(if_keyword), span })
                    if if_keyword.as_str() == "if" => {
                        self.advance();
                        let if_keyword = TokenTree { kind: TokenNode::Term(if_keyword), span };
                        render.push(if_keyword);
                        self.if_expr(render)?;
                    },
                    // Just an `@else`
                    _ => {
                        if let Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span }) = self.next() {
                            let block = self.block(block, span, render)?;
                            render.push(block);
                        } else {
                            return self.error("expected body for @else");
                        }
                    },
                }
                self.else_if_expr(render)
            },
            // We didn't find an `@else`; stop
            _ => Ok(()),
        }
    }

    /// Parses and renders an `@while` expression.
    ///
    /// The leading `@while` should already be consumed.
    fn while_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span }) => {
                    let block = self.block(block, span, render)?;
                    render.push(block);
                    break;
                },
                Some(token) => render.push(token),
                None => return self.error("unexpected end of @while expression"),
            }
        }
        Ok(())
    }

    /// Parses and renders a `@for` expression.
    ///
    /// The leading `@for` should already be consumed.
    fn for_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Term(in_keyword), span }) if in_keyword.as_str() == "in" => {
                    render.push(TokenTree { kind: TokenNode::Term(in_keyword), span });
                    break;
                },
                Some(token) => render.push(token),
                None => return self.error("unexpected end of @for expression"),
            }
        }
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), span }) => {
                    let block = self.block(block, span, render)?;
                    render.push(block);
                    break;
                },
                Some(token) => render.push(token),
                None => return self.error("unexpected end of @for expression"),
            }
        }
        Ok(())
    }

    /// Parses and renders a `@match` expression.
    ///
    /// The leading `@match` should already be consumed.
    fn match_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, body), span }) => {
                    let body = Parser {
                        in_attr: self.in_attr,
                        input: body.into_iter(),
                    }.match_arms(render)?;
                    render.push(TokenTree {
                        kind: TokenNode::Group(Delimiter::Brace, body),
                        span,
                    });
                    break;
                },
                Some(token) => render.push(token),
                None => return self.error("unexpected end of @match expression"),
            }
        }
        Ok(())
    }

    fn match_arms(&mut self, render: &mut Renderer) -> ParseResult<TokenStream> {
        let mut arms = Vec::new();
        while let Some(arm) = self.match_arm(render)? {
            arms.push(arm);
        }
        Ok(arms.into_iter().collect())
    }

    fn match_arm(&mut self, render: &mut Renderer) -> ParseResult<Option<TokenStream>> {
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
                let body = self.block(body, span, render)?;
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
                self.block(body.into_iter().collect(), Span::default(), render)?
            },
            None => return self.error("unexpected end of @match arm"),
        };
        Ok(Some(pat.into_iter().chain(iter::once(body)).collect()))
    }

    /// Parses and renders a `@let` expression.
    ///
    /// The leading `@let` should already be consumed.
    fn let_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        loop {
            match self.next() {
                Some(token @ TokenTree { kind: TokenNode::Op('=', _), .. }) => {
                    render.push(token);
                    break;
                },
                Some(token) => render.push(token),
                None => return self.error("unexpected end of @let expression"),
            }
        }
        loop {
            match self.next() {
                Some(token @ TokenTree { kind: TokenNode::Op(';', _), .. }) => {
                    render.push(token);
                    break;
                },
                Some(token) => render.push(token),
                None => return self.error("unexpected end of @let expression"),
            }
        }
        Ok(())
    }

    /// Parses and renders an element node.
    ///
    /// The element name should already be consumed.
    fn element(&mut self, name: &str, render: &mut Renderer) -> ParseResult<()> {
        if self.in_attr {
            return self.error("unexpected element, you silly bumpkin");
        }
        render.element_open_start(name);
        self.attrs(render)?;
        render.element_open_end();
        match self.peek() {
            Some(TokenTree { kind: TokenNode::Op(';', _), .. }) |
            Some(TokenTree { kind: TokenNode::Op('/', _), .. }) => {
                // Void element
                self.advance();
            },
            _ => {
                self.markup(render)?;
                render.element_close(name);
            },
        }
        Ok(())
    }

    /// Parses and renders the attributes of an element.
    fn attrs(&mut self, render: &mut Renderer) -> ParseResult<()> {
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
                    render.attribute_start(&name);
                    {
                        // Parse a value under an attribute context
                        let in_attr = mem::replace(&mut self.in_attr, true);
                        self.markup(render)?;
                        self.in_attr = in_attr;
                    }
                    render.attribute_end();
                },
                // Empty attribute
                (Ok(name), Some(TokenTree { kind: TokenNode::Op('?', _), span: question_span })) => {
                    self.commit(attempt);
                    match self.peek() {
                        // Toggle the attribute based on a boolean expression
                        Some(TokenTree {
                            kind: TokenNode::Group(Delimiter::Bracket, cond),
                            span: delim_span,
                        }) => {
                            self.advance();
                            render.push(TokenTree {
                                kind: TokenNode::Term(Term::intern("if")),
                                span: question_span,
                            });
                            // If the condition contains an opening brace `{`,
                            // wrap it in parentheses to avoid parse errors
                            if cond.clone().into_iter().any(|token| match token.kind {
                                TokenNode::Group(Delimiter::Brace, _) => true,
                                _ => false,
                            }) {
                                render.push(TokenTree {
                                    kind: TokenNode::Group(Delimiter::Parenthesis, cond),
                                    span: delim_span,
                                });
                            } else {
                                render.push(cond);
                            }
                            let body = {
                                let mut render = render.fork();
                                render.attribute_empty(&name);
                                render.into_stmts()
                            };
                            render.push(TokenTree {
                                kind: TokenNode::Group(Delimiter::Brace, body),
                                span: Span::default(),
                            });
                        },
                        // Write the attribute unconditionally
                        _ => render.attribute_empty(&name),
                    }
                },
                // Class shorthand
                (Err(_), Some(TokenTree { kind: TokenNode::Op('.', _), .. })) => {
                    self.commit(attempt);
                    let class_name = self.name()?;
                    match self.peek() {
                        // Toggle the class based on a boolean expression
                        Some(TokenTree { kind: TokenNode::Group(Delimiter::Bracket, cond), .. }) => {
                            self.advance();
                            classes_toggled.push((cond, class_name));
                        },
                        // Emit the class unconditionally
                        _ => classes_static.push(class_name),
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
            render.attribute_start("class");
            render.string(&classes_static.join(" "));
            for (i, (cond, mut class_name)) in classes_toggled.into_iter().enumerate() {
                // If a class comes first in the list, then it shouldn't be
                // prefixed by a space
                if i > 0 || !classes_static.is_empty() {
                    class_name = format!(" {}", class_name);
                }
                let body = {
                    let mut render = render.fork();
                    render.string(&class_name);
                    render.into_stmts()
                };
                render.emit_if(cond, body, None);
            }
            render.attribute_end();
        }
        if !ids.is_empty() {
            render.attribute_start("id");
            render.string(&ids.join(" "));
            render.attribute_end();
        }
        Ok(())
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
    fn block(&mut self, body: TokenStream, span: Span, render: &mut Renderer) -> ParseResult<TokenTree> {
        let mut render = render.fork();
        let mut parse = Parser {
            in_attr: self.in_attr,
            input: body.into_iter(),
        };
        parse.markups(&mut render)?;
        Ok(TokenTree {
            kind: TokenNode::Group(Delimiter::Brace, render.into_stmts()),
            span,
        })
    }
}
