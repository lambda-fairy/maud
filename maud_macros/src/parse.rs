use std::mem;
use std::rc::Rc;
use syntax::ast::{Delimited, Expr, ExprKind, Lit, LitKind, Stmt, TokenTree};
use syntax::ext::quote::rt::ToTokens;
use syntax::codemap::Span;
use syntax::errors::{DiagnosticBuilder, FatalError};
use syntax::ext::base::ExtCtxt;
use syntax::parse;
use syntax::parse::parser::Parser as RustParser;
use syntax::parse::token::{BinOpToken, DelimToken, Token, Lit as LitToken};
use syntax::parse::token::keywords;
use syntax::ptr::P;

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
macro_rules! modsep {
    () => (TokenTree::Token(_, Token::ModSep))
}
macro_rules! eq {
    () => (TokenTree::Token(_, Token::Eq))
}
macro_rules! not {
    () => (TokenTree::Token(_, Token::Not))
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
macro_rules! caret {
    () => (TokenTree::Token(_, Token::BinOp(BinOpToken::Caret)))
}
macro_rules! literal {
    () => (TokenTree::Token(_, Token::Literal(..)))
}
macro_rules! integer {
    () => (TokenTree::Token(_, Token::Literal(LitToken::Integer(_), _)))
}
macro_rules! ident {
    ($sp:pat, $x:pat) => (TokenTree::Token($sp, Token::Ident($x)))
}
macro_rules! substnt {
    ($sp:pat, $x:pat) => (TokenTree::Token($sp, Token::SubstNt($x)))
}
macro_rules! keyword {
    ($sp:pat, $x:ident) => (TokenTree::Token($sp, ref $x @ Token::Ident(..)))
}

pub fn parse(cx: &ExtCtxt, sp: Span, write: &[TokenTree], input: &[TokenTree])
    -> PResult<P<Expr>>
{
    let mut parser = Parser {
        in_attr: false,
        input: input,
        span: sp,
        render: Renderer::new(cx),
    };
    parser.markups()?;
    Ok(parser.into_render().into_expr(write.to_vec()))
}

pub fn split_comma<'a>(cx: &ExtCtxt, sp: Span, mac_name: &str, args: &'a [TokenTree])
    -> PResult<(&'a [TokenTree], &'a [TokenTree])>
{
    fn is_comma(t: &TokenTree) -> bool {
        match *t {
            TokenTree::Token(_, Token::Comma) => true,
            _ => false,
        }
    }
    match args.iter().position(is_comma) {
        Some(i) => Ok((&args[..i], &args[1+i..])),
        None => error!(cx, sp, &format!("expected two arguments to `{}!`", mac_name)),
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
        let mut parser = parse::tts_to_parser(self.render.cx.parse_sess, tts,
                                              self.render.cx.cfg.clone());
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
            [minus!(), ref tt @ literal!(), ..] => {
                self.shift(2);
                self.literal(tt, true)?;
            },
            [ref tt @ literal!(), ..] => {
                self.shift(1);
                self.literal(tt, false)?;
            },
            // If
            [at!(), keyword!(sp, k), ..] if k.is_keyword(keywords::If) => {
                self.shift(2);
                self.if_expr(sp)?;
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
            // Call
            [at!(), ident!(sp, name), ..] if name.name.as_str() == "call" => {
                self.shift(2);
                let func = self.splice(sp)?;
                self.render.emit_call(func);
            },
            // Splice
            [ref tt @ caret!(), ..] => {
                self.shift(1);
                let expr = self.splice(tt.get_span())?;
                self.render.splice(expr);
            },
            // Element
            [ident!(sp, _), ..] => {
                let name = self.name()?;
                self.element(sp, &name)?;
            },
            // Block
            [TokenTree::Delimited(_, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                {
                    // Parse the contents of the block, emitting the
                    // result inline
                    let mut i = &*d.tts;
                    mem::swap(&mut self.input, &mut i);
                    self.markups()?;
                    mem::swap(&mut self.input, &mut i);
                }
            },
            // ???
            _ => {
                if let [ref tt, ..] = *self.input {
                    parse_error!(self, tt.get_span(), "invalid syntax");
                } else {
                    parse_error!(self, self.span, "unexpected end of block");
                }
            },
        }
        Ok(())
    }

    /// Parses and renders a literal string or number.
    fn literal(&mut self, tt: &TokenTree, minus: bool) -> PResult<()> {
        let lit = self.with_rust_parser(vec![tt.clone()], RustParser::parse_lit)?;
        let s = lit_to_string(self.render.cx, lit, minus)?;
        self.render.string(&s);
        Ok(())
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
            [TokenTree::Token(sp, _), ..] | [TokenTree::Delimited(sp, _), ..] | [TokenTree::Sequence(sp, _), ..] => {
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
          open_span: sp,
          tts: expr,
          close_span: sp,
        })));
        Ok(body)
    }

    /// Parses and renders a `^splice`.
    ///
    /// The leading `^` should already be consumed.
    fn splice(&mut self, sp: Span) -> PResult<P<Expr>> {
        // First, munch a single token tree
        let prefix = match *self.input {
            [ref tt, ..] => {
                self.shift(1);
                tt.clone()
            },
            [] => parse_error!(self, sp, "expected expression for this splice"),
        };
        self.splice_with_prefix(prefix)
    }

    /// Parses and renders a `^splice`, given a prefix that we've already
    /// consumed.
    fn splice_with_prefix(&mut self, prefix: TokenTree) -> PResult<P<Expr>> {
        let mut tts = vec![prefix];
        loop { match *self.input {
            // Munch attribute lookups e.g. `^person.address.street`
            [ref dot @ dot!(), ref ident @ ident!(_, _), ..] => {
                self.shift(2);
                tts.push(dot.clone());
                tts.push(ident.clone());
            },
            // Munch tuple attribute lookups e.g. `^person.1.2`
            [ref dot @ dot!(), ref num @ integer!(), ..] => {
                self.shift(2);
                tts.push(dot.clone());
                tts.push(num.clone());
            },
            // Munch path lookups e.g. `^some_mod::Struct`
            [ref sep @ modsep!(), ref ident @ ident!(_, _), ..] => {
                self.shift(2);
                tts.push(sep.clone());
                tts.push(ident.clone());
            },
            // Munch function calls `()` and indexing operations `[]`
            [TokenTree::Delimited(sp, ref d), ..] if d.delim != DelimToken::Brace => {
                self.shift(1);
                tts.push(TokenTree::Delimited(sp, d.clone()));
            },
            _ => break,
        }}
        self.with_rust_parser(tts, RustParser::parse_expr)
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
        let mut classes = Vec::new();
        let mut ids = Vec::new();
        loop {
            let old_input = self.input;
            let maybe_name = self.name();
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
                    if let [ref tt @ eq!(), ..] = *self.input {
                        // Toggle the attribute based on a boolean expression
                        self.shift(1);
                        let cond = self.splice(tt.get_span())?;
                        // Silence "unnecessary parentheses" warnings
                        let cond = strip_outer_parens(cond).to_tokens(self.render.cx);
                        let body = {
                            let mut r = self.render.fork();
                            r.attribute_empty(&name);
                            r.into_stmts()
                        };
                        self.render.emit_if(cond, body, None);
                    } else {
                        // Write the attribute unconditionally
                        self.render.attribute_empty(&name);
                    }
                },
                (Err(_), &[dot!(), ident!(_, _), ..]) => {
                    // Class shorthand
                    self.shift(1);
                    classes.push(self.name()?);
                },
                (Err(_), &[pound!(), ident!(_, _), ..]) => {
                    // ID shorthand
                    self.shift(1);
                    ids.push(self.name()?);
                },
                _ => {
                    self.input = old_input;
                    break;
                },
            }
        }
        if !classes.is_empty() {
            self.render.attribute_start("class");
            self.render.string(&classes.join(" "));
            self.render.attribute_end();
        }
        if !ids.is_empty() {
            self.render.attribute_start("id");
            self.render.string(&ids.join(" "));
            self.render.attribute_end();
        }
        Ok(())
    }

    /// Parses a HTML element or attribute name.
    fn name(&mut self) -> PResult<String> {
        let mut s = match *self.input {
            [ident!(_, name), ..] => {
                self.shift(1);
                String::from(&name.name.as_str() as &str)
            },
            _ => return Err(FatalError),
        };
        while let [minus!(), ident!(_, name), ..] = *self.input {
            self.shift(2);
            s.push('-');
            s.push_str(&name.name.as_str());
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

/// Converts a literal to a string.
fn lit_to_string(cx: &ExtCtxt, lit: Lit, minus: bool) -> PResult<String> {
    let mut result = String::new();
    if minus {
        result.push('-');
    }
    match lit.node {
        LitKind::Str(s, _) => result.push_str(&s),
        LitKind::ByteStr(..) | LitKind::Byte(..) => {
            error!(cx, lit.span, "cannot splice binary data");
        },
        LitKind::Char(c) => result.push(c),
        LitKind::Int(x, _) => result.push_str(&x.to_string()),
        LitKind::Float(s, _) | LitKind::FloatUnsuffixed(s) => result.push_str(&s),
        LitKind::Bool(b) => result.push_str(if b { "true" } else { "false" }),
    };
    Ok(result)
}

/// If the expression is wrapped in parentheses, strip them off.
fn strip_outer_parens(expr: P<Expr>) -> P<Expr> {
    expr.and_then(|expr| match expr {
        Expr { node: ExprKind::Paren(inner), .. } => inner,
        expr => P(expr),
    })
}
