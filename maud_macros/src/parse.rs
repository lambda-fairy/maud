use std::mem;
use std::rc::Rc;
use syntax::ast::{Expr, LitKind, Stmt};
use syntax::ext::quote::rt::ToTokens;
use syntax::codemap::Span;
use syntax::errors::{DiagnosticBuilder, FatalError};
use syntax::ext::base::ExtCtxt;
use syntax::fold::Folder;
use syntax::parse;
use syntax::parse::parser::Parser as RustParser;
use syntax::parse::token::{BinOpToken, DelimToken, Nonterminal, Token};
use syntax::print::pprust;
use syntax::ptr::P;
use syntax::symbol::keywords;
use syntax::tokenstream::{Delimited, TokenTree};

use super::render::Renderer;
use super::PResult;

macro_rules! error {
    ($cx:expr, $sp:expr, $msg:expr) => ({
        $cx.span_err($sp, $msg);
        return Err(::syntax::errors::FatalError);
    })
}
macro_rules! parse_error {
    ($self_:expr, $sp:expr, $msg:expr) => (error!($self_.render.cx, $sp, $msg))
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

pub fn parse(cx: &ExtCtxt, sp: Span, input: &[TokenTree]) -> PResult<P<Expr>> {
    let input = FlattenNtFolder.fold_tts(input);
    let mut parser = Parser {
        in_attr: false,
        input: &input,
        span: sp,
        render: Renderer::new(cx),
    };
    parser.markups()?;
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = pprust::tts_to_string(&input).len();
    Ok(parser.into_render().into_expr(size_hint))
}

struct FlattenNtFolder;

impl Folder for FlattenNtFolder {
    fn fold_tt(&mut self, mut tt: &TokenTree) -> TokenTree {
        while let TokenTree::Token(_, Token::Interpolated(ref nt)) = *tt {
            if let Nonterminal::NtTT(ref sub_tt) = **nt {
                tt = sub_tt;
            } else {
                break;
            }
        }
        tt.clone()
    }
}

struct Parser<'cx, 'a: 'cx, 'i> {
    in_attr: bool,
    input: &'i [TokenTree],
    span: Span,
    render: Renderer<'cx, 'a>,
}

impl<'cx, 'a, 'i> Parser<'cx, 'a, 'i> {
    /// Finalizes the `Parser`, returning the `Renderer` underneath.
    fn into_render(self) -> Renderer<'cx, 'a> {
        let Parser { render, .. } = self;
        render
    }

    /// Consumes `n` items from the input.
    fn shift(&mut self, n: usize) {
        self.input = &self.input[n..];
    }

    /// Constructs a Rust AST parser from the given token tree.
    fn with_rust_parser<F, T>(&self, tts: Vec<TokenTree>, callback: F) -> PResult<T> where
        F: FnOnce(&mut RustParser<'cx>) -> Result<T, DiagnosticBuilder<'cx>>
    {
        let mut parser = parse::tts_to_parser(self.render.cx.parse_sess, tts);
        let result = callback(&mut parser).map_err(|mut e| { e.emit(); FatalError });
        // Make sure all tokens were consumed
        if parser.token != Token::Eof {
            let token = parser.this_token_to_string();
            self.render.cx.span_err(parser.span,
                                    &format!("unexpected token: `{}`", token));
        }
        result
    }

    /// Parses and renders multiple blocks of markup.
    fn markups(&mut self) -> PResult<()> {
        loop {
            match *self.input {
                [] => return Ok(()),
                [semi!(), ..] => self.shift(1),
                [_, ..] => self.markup()?,
            }
        }
    }

    /// Parses and renders a single block of markup.
    fn markup(&mut self) -> PResult<()> {
        match *self.input {
            // Literal
            [ref tt @ literal!(), ..] => {
                self.shift(1);
                self.literal(tt)?;
            },
            // If
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::If) => {
                self.shift(2);
                self.if_expr(sp)?;
            },
            // While
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::While) => {
                self.shift(2);
                self.while_expr(sp)?;
            },
            // For
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::For) => {
                self.shift(2);
                self.for_expr(sp)?;
            },
            // Match
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::Match) => {
                self.shift(2);
                self.match_expr(sp)?;
            },
            // Let
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::Let) => {
                self.shift(2);
                self.let_expr(sp)?;
            }
            // Element
            [ident!(sp, _), ..] => {
                let name = self.namespaced_name().unwrap();
                self.element(sp, &name)?;
            },
            // Splice
            [TokenTree::Delimited(_, ref d), ..] if d.delim == DelimToken::Paren => {
                self.shift(1);
                let expr = self.with_rust_parser(d.tts.clone(), RustParser::parse_expr)?;
                self.render.splice(expr);
            }
            // Block
            [TokenTree::Delimited(_, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                {
                    // Parse the contents of the block, emitting the
                    // result inline
                    let mut i = &d.tts[..];
                    mem::swap(&mut self.input, &mut i);
                    self.markups()?;
                    mem::swap(&mut self.input, &mut i);
                }
            },
            // ???
            _ => {
                if let [ref tt, ..] = *self.input {
                    parse_error!(self, tt.span(), "invalid syntax");
                } else {
                    parse_error!(self, self.span, "unexpected end of block");
                }
            },
        }
        Ok(())
    }

    /// Parses and renders a literal string.
    fn literal(&mut self, tt: &TokenTree) -> PResult<()> {
        let lit = self.with_rust_parser(vec![tt.clone()], RustParser::parse_lit)?;
        if let LitKind::Str(s, _) = lit.node {
            self.render.string(&s.as_str());
            Ok(())
        } else {
            parse_error!(self, lit.span, "literal strings must be surrounded by quotes (\"like this\")")
        }
    }

    /// Parses and renders an `@if` expression.
    ///
    /// The leading `@if` should already be consumed.
    fn if_expr(&mut self, sp: Span) -> PResult<()> {
        // Parse the initial if
        let mut if_cond = vec![];
        let if_body;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                if_body = self.block(sp, &d.tts)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                if_cond.push(tt.clone());
            },
            [] => parse_error!(self, sp, "expected body for this @if"),
        }}
        // Parse the (optional) @else
        let else_body = match *self.input {
            [at!(), keyword!(_, k), ..] if k.is_keyword(keywords::Else) => {
                self.shift(2);
                match *self.input {
                    [keyword!(sp, k), ..] if k.is_keyword(keywords::If) => {
                        self.shift(1);
                        let else_body = {
                            // Parse an if expression, but capture the result
                            // rather than emitting it right away
                            let mut r = self.render.fork();
                            mem::swap(&mut self.render, &mut r);
                            self.if_expr(sp)?;
                            mem::swap(&mut self.render, &mut r);
                            r.into_stmts()
                        };
                        Some(else_body)
                    },
                    [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                        self.shift(1);
                        Some(self.block(sp, &d.tts)?)
                    },
                    _ => parse_error!(self, sp, "expected body for this @else"),
                }
            },
            _ => None,
        };
        self.render.emit_if(if_cond, if_body, else_body);
        Ok(())
    }

    /// Parses and renders an `@while` expression.
    ///
    /// The leading `@while` should already be consumed.
    fn while_expr(&mut self, sp: Span) -> PResult<()> {
        let mut cond = vec![];
        let body;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                body = self.block(sp, &d.tts)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                cond.push(tt.clone());
            },
            [] => parse_error!(self, sp, "expected body for this @while"),
        }}
        self.render.emit_while(cond, body);
        Ok(())
    }

    /// Parses and renders a `@for` expression.
    ///
    /// The leading `@for` should already be consumed.
    fn for_expr(&mut self, sp: Span) -> PResult<()> {
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
            _ => parse_error!(self, sp, "invalid @for"),
        }}
        let pattern = self.with_rust_parser(pattern, RustParser::parse_pat)?;
        let mut iterable = vec![];
        let body;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                body = self.block(sp, &d.tts)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                iterable.push(tt.clone());
            },
            _ => parse_error!(self, sp, "invalid @for"),
        }}
        let iterable = self.with_rust_parser(iterable, RustParser::parse_expr)?;
        self.render.emit_for(pattern, iterable, body);
        Ok(())
    }

    /// Parses and renders a `@match` expression.
    ///
    /// The leading `@match` should already be consumed.
    fn match_expr(&mut self, sp: Span) -> PResult<()> {
        // Parse the initial match
        let mut match_var = vec![];
        let match_bodies;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                match_bodies = Parser {
                    in_attr: self.in_attr,
                    input: &d.tts,
                    span: sp,
                    render: self.render.fork(),
                }.match_bodies()?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                match_var.push(tt.clone());
            },
            [] => parse_error!(self, sp, "expected body for this @match"),
        }}
        let match_var = self.with_rust_parser(match_var, RustParser::parse_expr)?;
        self.render.emit_match(match_var, match_bodies);
        Ok(())
    }

    fn match_bodies(&mut self) -> PResult<Vec<TokenTree>> {
        let mut bodies = Vec::new();
        loop { match *self.input {
            [] => break,
            [ref tt @ comma!(), ..] => {
                self.shift(1);
                bodies.push(tt.clone());
            },
            [TokenTree::Token(sp, _), ..] | [TokenTree::Delimited(sp, _), ..] => {
                bodies.append(&mut self.match_body(sp)?);
            },
        }}
        Ok(bodies)
    }

    fn match_body(&mut self, sp: Span) -> PResult<Vec<TokenTree>> {
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
            _ => parse_error!(self, sp, "invalid @match pattern"),
        }}
        let mut expr = Vec::new();
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                if expr.is_empty() {
                    self.shift(1);
                    expr = self.block(sp, &d.tts)?.to_tokens(self.render.cx);
                    break;
                } else {
                    self.shift(1);
                    expr.push(TokenTree::Delimited(sp, d.clone()));
                }
            },
            [comma!(), ..] | [] => {
                if expr.is_empty() {
                    parse_error!(self, sp, "expected body for this @match arm");
                } else {
                    expr = self.block(sp, &expr)?.to_tokens(self.render.cx);
                    break;
                }
            },
            [ref tt, ..] => {
                self.shift(1);
                expr.push(tt.clone());
            },
        }}
        body.push(TokenTree::Delimited(sp, Rc::new(Delimited {
            delim: DelimToken::Brace,
            tts: expr,
        })));
        Ok(body)
    }

    /// Parses and renders a `@let` expression.
    ///
    /// The leading `@let` should already be consumed.
    fn let_expr(&mut self, sp: Span) -> PResult<()> {
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
            _ => parse_error!(self, sp, "invalid @let"),
        }}
        let pattern = self.with_rust_parser(pattern, RustParser::parse_pat)?;
        let mut rhs = vec![];
        let body;
        loop { match *self.input {
            [TokenTree::Delimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                body = self.block(sp, &d.tts)?;
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                rhs.push(tt.clone());
            },
            _ => parse_error!(self, sp, "invalid @let"),
        }}
        let rhs = self.with_rust_parser(rhs, RustParser::parse_expr)?;
        self.render.emit_let(pattern, rhs, body);
        Ok(())
    }

    /// Parses and renders an element node.
    ///
    /// The element name should already be consumed.
    fn element(&mut self, sp: Span, name: &str) -> PResult<()> {
        if self.in_attr {
            parse_error!(self, sp, "unexpected element, you silly bumpkin");
        }
        self.render.element_open_start(name);
        self.attrs()?;
        self.render.element_open_end();
        if let [slash!(), ..] = *self.input {
            self.shift(1);
        } else {
            self.markup()?;
            self.render.element_close(name);
        }
        Ok(())
    }

    /// Parses and renders the attributes of an element.
    fn attrs(&mut self) -> PResult<()> {
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
                    self.render.attribute_start(&name);
                    {
                        // Parse a value under an attribute context
                        let mut in_attr = true;
                        mem::swap(&mut self.in_attr, &mut in_attr);
                        self.markup()?;
                        mem::swap(&mut self.in_attr, &mut in_attr);
                    }
                    self.render.attribute_end();
                },
                (Ok(name), &[question!(), ..]) => {
                    // Empty attribute
                    self.shift(1);
                    match *self.input {
                        [TokenTree::Delimited(_, ref d), ..] if d.delim == DelimToken::Bracket => {
                            // Toggle the attribute based on a boolean expression
                            self.shift(1);
                            let cond = self.with_rust_parser(d.tts.clone(), RustParser::parse_expr)?;
                            let cond = cond.to_tokens(self.render.cx);
                            let body = {
                                let mut r = self.render.fork();
                                r.attribute_empty(&name);
                                r.into_stmts()
                            };
                            self.render.emit_if(cond, body, None);
                        },
                        _ => {
                            // Write the attribute unconditionally
                            self.render.attribute_empty(&name);
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
                            let cond = self.with_rust_parser(d.tts.clone(), RustParser::parse_expr)?;
                            let cond = cond.to_tokens(self.render.cx);
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
            self.render.attribute_start("class");
            self.render.string(&classes_static.join(" "));
            for (i, (cond, mut class_name)) in classes_toggled.into_iter().enumerate() {
                // If a class comes first in the list, then it shouldn't be
                // prefixed by a space
                if i > 0 || !classes_static.is_empty() {
                    class_name = format!(" {}", class_name);
                }
                let body = {
                    let mut r = self.render.fork();
                    r.string(&class_name);
                    r.into_stmts()
                };
                self.render.emit_if(cond, body, None);
            }
            self.render.attribute_end();
        }
        if !ids.is_empty() {
            self.render.attribute_start("id");
            self.render.string(&ids.join(" "));
            self.render.attribute_end();
        }
        Ok(())
    }

    /// Parses an identifier, without dealing with namespaces.
    fn name(&mut self) -> PResult<String> {
        let mut s = match *self.input {
            [ident!(_, name), ..] => {
                self.shift(1);
                String::from(&name.name.as_str() as &str)
            },
            _ => return Err(FatalError),
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
    fn namespaced_name(&mut self) -> PResult<String> {
        let mut s = self.name()?;
        if let [colon!(), ident!(_, _), ..] = *self.input {
            self.shift(1);
            s.push(':');
            s.push_str(&self.name().unwrap());
        }
        Ok(s)
    }

    /// Parses the given token tree, returning a vector of statements.
    fn block(&mut self, sp: Span, tts: &[TokenTree]) -> PResult<Vec<Stmt>> {
        let mut parse = Parser {
            in_attr: self.in_attr,
            input: tts,
            span: sp,
            render: self.render.fork(),
        };
        parse.markups()?;
        Ok(parse.into_render().into_stmts())
    }
}
