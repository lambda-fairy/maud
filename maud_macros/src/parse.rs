use proc_macro::{Delimiter, Literal, TokenNode, TokenStream, TokenTree, TokenTreeIter};
use std::mem;

use literalext::LiteralExt;

use super::render::Renderer;
use super::ParseResult;

pub fn parse(input: TokenStream) -> ParseResult<TokenStream> {
    let mut render = Renderer::new();
    let _ = Parser {
        in_attr: false,
        input: Lookahead::new(input.clone()),
    }.markups(&mut render);
    /*
    Parser {
        in_attr: false,
        input: Lookahead::new(input.clone()),
    }.markups(&mut render)?;
    */
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    Ok(render.into_expr(size_hint))
}

struct Parser {
    in_attr: bool,
    input: Lookahead<TokenTree>,
}

impl Parser {
    fn next(&mut self) -> Option<TokenTree> {
        self.input.next()
    }

    fn peek(&mut self) -> Option<TokenTree> {
        self.input.peek()
    }

    fn advance(&mut self) {
        self.next();
    }

    /// Attaches an error message to the span and returns `Err`.
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
                    Some(TokenTree { kind: TokenNode::Term(term), .. }) => match term.as_str() {
                        "if" => self.if_expr(render)?,
                        "while" => self.while_expr(render)?,
                        "for" => self.for_expr(render)?,
                        "match" => self.match_expr(render)?,
                        "let" => self.let_expr(render)?,
                        other => return self.error(format!("unknown keyword `@{}`", other)),
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
            TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), .. } => {
                self.advance();
                Parser {
                    in_attr: self.in_attr,
                    input: Lookahead::new(block),
                }.markups(render)?;
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
        self.error("unimplemented")
    }

    /// Parses and renders an `@while` expression.
    ///
    /// The leading `@while` should already be consumed.
    fn while_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        self.error("unimplemented")
    }

    /// Parses and renders a `@for` expression.
    ///
    /// The leading `@for` should already be consumed.
    fn for_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        self.error("unimplemented")
    }

    /// Parses and renders a `@match` expression.
    ///
    /// The leading `@match` should already be consumed.
    fn match_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        self.error("unimplemented")
    }

    fn match_bodies(&mut self, render: &mut Renderer) -> ParseResult<Vec<TokenTree>> {
        self.error("unimplemented")
    }

    fn match_body(&mut self, render: &mut Renderer) -> ParseResult<Vec<TokenTree>> {
        self.error("unimplemented")
    }

    /// Parses and renders a `@let` expression.
    ///
    /// The leading `@let` should already be consumed.
    fn let_expr(&mut self, render: &mut Renderer) -> ParseResult<()> {
        let mut pat = Vec::new();
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Op('=', _), .. }) => break,
                Some(token) => pat.push(token),
                None => return self.error("unexpected end of @let expression"),
            }
        }
        let mut expr = Vec::new();
        let body;
        loop {
            match self.next() {
                Some(TokenTree { kind: TokenNode::Group(Delimiter::Brace, block), .. }) => {
                    body = self.block(block, render)?;
                    break;
                },
                Some(token) => expr.push(token),
                None => return self.error("unexpected end of @let expression"),
            }
        }
        render.emit_let(pat.into_iter().collect(), expr.into_iter().collect(), body);
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
            let start_position = self.input.save();
            let maybe_name = self.namespaced_name();
            let token_after = self.next();
            match (maybe_name, token_after) {
                // Non-empty attribute
                (Ok(name), Some(TokenTree { kind: TokenNode::Op('=', _), .. })) => {
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
                (Ok(name), Some(TokenTree { kind: TokenNode::Op('?', _), .. })) => match self.peek() {
                    // Toggle the attribute based on a boolean expression
                    Some(TokenTree { kind: TokenNode::Group(Delimiter::Bracket, cond), .. }) => {
                        self.advance();
                        let body = {
                            let mut render = render.fork();
                            render.attribute_empty(&name);
                            render.into_stmts()
                        };
                        render.emit_if(cond, body, None);
                    },
                    // Write the attribute unconditionally
                    _ => render.attribute_empty(&name),
                },
                // Class shorthand
                (Err(_), Some(TokenTree { kind: TokenNode::Op('.', _), .. })) => {
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
                    ids.push(self.name()?);
                },
                // If it's not a valid attribute, backtrack and bail out
                _ => {
                    self.input.restore(start_position);
                    break;
                },
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

    /// Parses the given token tree, returning a vector of statements.
    fn block(&mut self, body: TokenStream, render: &mut Renderer) -> ParseResult<TokenStream> {
        let mut render = render.fork();
        let mut parse = Parser {
            in_attr: self.in_attr,
            input: Lookahead::new(body),
        };
        parse.markups(&mut render)?;
        Ok(render.into_stmts())
    }
}

struct Lookahead<T> {
    buffer: Vec<T>,
    index: usize,
}

impl<T> Lookahead<T> {
    fn new<I: IntoIterator<Item=T>>(items: I) -> Self {
        Lookahead {
            buffer: items.into_iter().collect(),
            index: 0,
        }
    }

    fn save(&self) -> Position {
        Position { index: self.index }
    }

    fn restore(&mut self, Position { index }: Position) {
        self.index = index;
    }
}

impl<T> Lookahead<T> where T: Clone {
    fn peek(&mut self) -> Option<T> {
        let position = self.save();
        let result = self.next();
        self.restore(position);
        result
    }
}

impl<T> Iterator for Lookahead<T> where T: Clone {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let result = self.buffer.get(self.index).cloned();
        if result.is_some() {
            self.index += 1;
        }
        result
    }
}

struct Position {
    index: usize,
}
