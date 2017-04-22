use std::mem;
use syntax::ast::LitKind;
use syntax::codemap::Span;
use syntax::ext::base::ExtCtxt;
use syntax::parse;
use syntax::parse::token::{BinOpToken, DelimToken, Token};
use syntax::print::pprust;
use syntax::symbol::keywords;
use syntax::tokenstream::{Delimited, TokenStream, TokenTree};

use super::render::Renderer;
use super::ParseResult;

macro_rules! error {
    ($cx:expr, $sp:expr, $msg:expr) => ({
        $cx.span_err($sp, $msg);
        return Err(());
    })
}

macro_rules! at {
    () => (TokenTree::Token(_, Token::At))
}
macro_rules! dot {
    () => (TokenTree::Token(_, Token::Dot))
}
macro_rules! eq {
    () => (TokenTree::Token(_, Token::Eq))
}
macro_rules! pound {
    () => (TokenTree::Token(_, Token::Pound))
}
macro_rules! question {
    () => (TokenTree::Token(_, Token::Question))
}
macro_rules! semi {
    () => (TokenTree::Token(_, Token::Semi))
}
macro_rules! colon {
    () => (TokenTree::Token(_, Token::Colon))
}
macro_rules! comma {
    () => (TokenTree::Token(_, Token::Comma))
}
macro_rules! fat_arrow {
    () => (TokenTree::Token(_, Token::FatArrow))
}
macro_rules! minus {
    () => (TokenTree::Token(_, Token::BinOp(BinOpToken::Minus)))
}
macro_rules! slash {
    () => (TokenTree::Token(_, Token::BinOp(BinOpToken::Slash)))
}
macro_rules! literal {
    () => (TokenTree::Token(_, Token::Literal(..)))
}
macro_rules! ident {
    ($sp:pat, $x:pat) => (TokenTree::Token($sp, Token::Ident($x)))
}
macro_rules! keyword {
    ($sp:pat, $x:ident) => (TokenTree::Token($sp, ref $x @ Token::Ident(..)))
}

pub fn parse(cx: &ExtCtxt, sp: Span, input: &[TokenTree]) -> ParseResult<Vec<TokenTree>> {
    let mut render = Renderer::new(cx);
    Parser {
        cx,
        in_attr: false,
        input: input,
        span: sp,
    }.markups(&mut render)?;
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = pprust::tts_to_string(input).len();
    Ok(render.into_expr(size_hint).into_trees().collect())
}

struct Parser<'cx, 'a: 'cx, 'i> {
    cx: &'cx ExtCtxt<'a>,
    in_attr: bool,
    input: &'i [TokenTree],
    span: Span,
}

impl<'cx, 'a, 'i> Parser<'cx, 'a, 'i> {
    /// Consumes `n` items from the input.
    fn shift(&mut self, n: usize) {
        self.input = &self.input[n..];
    }

    /// Parses and renders multiple blocks of markup.
    fn markups(&mut self, render: &mut Renderer) -> ParseResult<()> {
        loop {
            match *self.input {
                [] => return Ok(()),
                [semi!(), ..] => self.shift(1),
                [_, ..] => self.markup(render)?,
            }
        }
    }

    /// Parses and renders a single block of markup.
    fn markup(&mut self, render: &mut Renderer) -> ParseResult<()> {
        match *self.input {
            // Literal
            [ref tt @ literal!(), ..] => {
                self.shift(1);
                self.literal(tt, render)?;
            },
            // If
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::If) => {
                self.shift(2);
                self.if_expr(sp, render)?;
            },
            // While
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::While) => {
                self.shift(2);
                self.while_expr(sp, render)?;
            },
            // For
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::For) => {
                self.shift(2);
                self.for_expr(sp, render)?;
            },
            // Match
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::Match) => {
                self.shift(2);
                self.match_expr(sp, render)?;
            },
            // Let
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::Let) => {
                self.shift(2);
                self.let_expr(sp, render)?;
            }
            // Element
            [ident!(sp, _), ..] => {
                let name = self.namespaced_name().unwrap();
                self.element(sp, &name, render)?;
            },
            // Splice
            [TokenTree::Delimited(_, ref d), ..] if d.delim == DelimToken::Paren => {
                self.shift(1);
                render.splice(d.stream());
            }
            // Block
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                Parser {
                    cx: self.cx,
                    in_attr: self.in_attr,
                    input: &d.stream().into_trees().collect::<Vec<_>>(),
                    span: sp,
                }.markups(render)?;
            },
            // ???
            _ => {
                if let [ref tt, ..] = *self.input {
                    error!(self.cx, tt.span(), "invalid syntax");
                } else {
                    error!(self.cx, self.span, "unexpected end of block");
                }
            },
        }
        Ok(())
    }

    /// Parses and renders a literal string.
    fn literal(&mut self, tt: &TokenTree, render: &mut Renderer) -> ParseResult<()> {
        let mut rust_parser = parse::stream_to_parser(self.cx.parse_sess, tt.clone().into());
        let lit = rust_parser.parse_lit().map_err(|mut e| e.emit())?;
        if let LitKind::Str(s, _) = lit.node {
            render.string(&s.as_str());
            Ok(())
        } else {
            error!(self.cx, lit.span, "literal strings must be surrounded by quotes (\"like this\")")
        }
    }

    /// Parses and renders an `@if` expression.
    ///
    /// The leading `@if` should already be consumed.
    fn if_expr(&mut self, sp: Span, render: &mut Renderer) -> ParseResult<()> {
        // Parse the initial if
        let mut if_cond = vec![];
        let if_body;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                if_body = self.block(sp, d.stream(), render)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                if_cond.push(tt.clone());
            },
            [] => error!(self.cx, sp, "expected body for this @if"),
        }}
        // Parse the (optional) @else
        let else_body = match *self.input {
            [at!(), keyword!(_, k), ..] if k.is_keyword(keywords::Else) => {
                self.shift(2);
                match *self.input {
                    [keyword!(sp, k), ..] if k.is_keyword(keywords::If) => {
                        self.shift(1);
                        let mut render = render.fork();
                        self.if_expr(sp, &mut render)?;
                        Some(render.into_stmts())
                    },
                    [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                        self.shift(1);
                        Some(self.block(sp, d.stream(), render)?)
                    },
                    _ => error!(self.cx, sp, "expected body for this @else"),
                }
            },
            _ => None,
        };
        render.emit_if(if_cond.into_iter().collect(), if_body, else_body);
        Ok(())
    }

    /// Parses and renders an `@while` expression.
    ///
    /// The leading `@while` should already be consumed.
    fn while_expr(&mut self, sp: Span, render: &mut Renderer) -> ParseResult<()> {
        let mut cond = vec![];
        let body;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                body = self.block(sp, d.stream(), render)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                cond.push(tt.clone());
            },
            [] => error!(self.cx, sp, "expected body for this @while"),
        }}
        render.emit_while(cond.into_iter().collect(), body);
        Ok(())
    }

    /// Parses and renders a `@for` expression.
    ///
    /// The leading `@for` should already be consumed.
    fn for_expr(&mut self, sp: Span, render: &mut Renderer) -> ParseResult<()> {
        let mut pattern = vec![];
        loop { match *self.input {
            [keyword!(_, k), ..] if k.is_keyword(keywords::In) => {
                self.shift(1);
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                pattern.push(tt.clone());
            },
            _ => error!(self.cx, sp, "invalid @for"),
        }}
        let mut iterable = vec![];
        let body;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                body = self.block(sp, d.stream(), render)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                iterable.push(tt.clone());
            },
            _ => error!(self.cx, sp, "invalid @for"),
        }}
        render.emit_for(pattern.into_iter().collect(), iterable.into_iter().collect(), body);
        Ok(())
    }

    /// Parses and renders a `@match` expression.
    ///
    /// The leading `@match` should already be consumed.
    fn match_expr(&mut self, sp: Span, render: &mut Renderer) -> ParseResult<()> {
        // Parse the initial match
        let mut match_var = vec![];
        let match_bodies;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                match_bodies = Parser {
                    cx: self.cx,
                    in_attr: self.in_attr,
                    input: &d.stream().into_trees().collect::<Vec<_>>(),
                    span: sp,
                }.match_bodies(render)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                match_var.push(tt.clone());
            },
            [] => error!(self.cx, sp, "expected body for this @match"),
        }}
        render.emit_match(match_var.into_iter().collect(), match_bodies.into_iter().collect());
        Ok(())
    }

    fn match_bodies(&mut self, render: &mut Renderer) -> ParseResult<Vec<TokenTree>> {
        let mut bodies = Vec::new();
        loop { match *self.input {
            [] => break,
            [ref tt @ comma!(), ..] => {
                self.shift(1);
                bodies.push(tt.clone());
            },
            [ref tt, ..] => bodies.append(&mut self.match_body(tt.span(), render)?),
        }}
        Ok(bodies)
    }

    fn match_body(&mut self, sp: Span, render: &mut Renderer) -> ParseResult<Vec<TokenTree>> {
        let mut body = vec![];
        loop { match *self.input {
            [ref tt @ fat_arrow!(), ..] => {
                self.shift(1);
                body.push(tt.clone());
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                body.push(tt.clone());
            },
            _ => error!(self.cx, sp, "invalid @match pattern"),
        }}
        let mut expr = Vec::new();
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                if expr.is_empty() {
                    self.shift(1);
                    expr = self.block(sp, d.stream(), render)?.into_trees().collect();
                    break;
                } else {
                    self.shift(1);
                    expr.push(TokenTree::Delimited(sp, d.clone()));
                }
            },
            [comma!(), ..] | [] => {
                if expr.is_empty() {
                    error!(self.cx, sp, "expected body for this @match arm");
                } else {
                    expr = self.block(sp, expr.into_iter().collect(), render)?.into_trees().collect();
                    break;
                }
            },
            [ref tt, ..] => {
                self.shift(1);
                expr.push(tt.clone());
            },
        }}
        body.push(TokenTree::Delimited(sp, Delimited {
            delim: DelimToken::Brace,
            tts: expr.into_iter().collect::<TokenStream>().into(),
        }));
        Ok(body)
    }

    /// Parses and renders a `@let` expression.
    ///
    /// The leading `@let` should already be consumed.
    fn let_expr(&mut self, sp: Span, render: &mut Renderer) -> ParseResult<()> {
        let mut pattern = vec![];
        loop { match *self.input {
            [eq!(), ..] => {
                self.shift(1);
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                pattern.push(tt.clone());
            },
            _ => error!(self.cx, sp, "invalid @let"),
        }}
        let mut rhs = vec![];
        let body;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                body = self.block(sp, d.stream(), render)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                rhs.push(tt.clone());
            },
            _ => error!(self.cx, sp, "invalid @let"),
        }}
        render.emit_let(pattern.into_iter().collect(), rhs.into_iter().collect(), body);
        Ok(())
    }

    /// Parses and renders an element node.
    ///
    /// The element name should already be consumed.
    fn element(&mut self, sp: Span, name: &str, render: &mut Renderer) -> ParseResult<()> {
        if self.in_attr {
            error!(self.cx, sp, "unexpected element, you silly bumpkin");
        }
        render.element_open_start(name);
        self.attrs(render)?;
        render.element_open_end();
        if let [slash!(), ..] = *self.input {
            self.shift(1);
        } else {
            self.markup(render)?;
            render.element_close(name);
        }
        Ok(())
    }

    /// Parses and renders the attributes of an element.
    fn attrs(&mut self, render: &mut Renderer) -> ParseResult<()> {
        let mut classes_static = Vec::new();
        let mut classes_toggled = Vec::new();
        let mut ids = Vec::new();
        loop {
            let old_input = self.input;
            let maybe_name = self.namespaced_name();
            match (maybe_name, self.input) {
                (Ok(name), &[eq!(), ..]) => {
                    // Non-empty attribute
                    self.shift(1);
                    render.attribute_start(&name);
                    {
                        // Parse a value under an attribute context
                        let mut in_attr = true;
                        mem::swap(&mut self.in_attr, &mut in_attr);
                        self.markup(render)?;
                        mem::swap(&mut self.in_attr, &mut in_attr);
                    }
                    render.attribute_end();
                },
                (Ok(name), &[question!(), ..]) => {
                    // Empty attribute
                    self.shift(1);
                    match *self.input {
                        [TokenTree::Delimited(_, ref d), ..] if d.delim == DelimToken::Bracket => {
                            // Toggle the attribute based on a boolean expression
                            self.shift(1);
                            let cond = d.stream();
                            let body = {
                                let mut render = render.fork();
                                render.attribute_empty(&name);
                                render.into_stmts()
                            };
                            render.emit_if(cond, body, None);
                        },
                        _ => {
                            // Write the attribute unconditionally
                            render.attribute_empty(&name);
                        },
                    }
                },
                (Err(_), &[dot!(), ident!(_, _), ..]) => {
                    // Class shorthand
                    self.shift(1);
                    let class_name = self.name().unwrap();
                    match *self.input {
                        [TokenTree::Delimited(_, ref d), ..] if d.delim == DelimToken::Bracket => {
                            // Toggle the class based on a boolean expression
                            self.shift(1);
                            let cond = d.stream();
                            classes_toggled.push((cond, class_name));
                        },
                        // Emit the class unconditionally
                        _ => classes_static.push(class_name),
                    }
                },
                (Err(_), &[pound!(), ident!(_, _), ..]) => {
                    // ID shorthand
                    self.shift(1);
                    ids.push(self.name().unwrap());
                },
                _ => {
                    self.input = old_input;
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
        let mut s = match *self.input {
            [ident!(_, name), ..] => {
                self.shift(1);
                String::from(&name.name.as_str() as &str)
            },
            _ => return Err(()),
        };
        let mut expect_ident = false;
        loop {
            expect_ident = match *self.input {
                [minus!(), ..] => {
                    self.shift(1);
                    s.push('-');
                    true
                },
                [ident!(_, name), ..] if expect_ident => {
                    self.shift(1);
                    s.push_str(&name.name.as_str());
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
        if let [colon!(), ident!(_, _), ..] = *self.input {
            self.shift(1);
            s.push(':');
            s.push_str(&self.name().unwrap());
        }
        Ok(s)
    }

    /// Parses the given token tree, returning a vector of statements.
    fn block(&mut self, sp: Span, tts: TokenStream, render: &mut Renderer) -> ParseResult<TokenStream> {
        let mut render = render.fork();
        let mut parse = Parser {
            cx: self.cx,
            in_attr: self.in_attr,
            input: &tts.into_trees().collect::<Vec<_>>(),
            span: sp,
        };
        parse.markups(&mut render)?;
        Ok(render.into_stmts())
    }
}
