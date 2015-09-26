use std::mem;
use syntax::ast::{Expr, ExprParen, Lit, Stmt, TokenTree, TtDelimited, TtToken};
use syntax::ext::quote::rt::ToTokens;
use syntax::codemap::Span;
use syntax::diagnostic::FatalError;
use syntax::ext::base::ExtCtxt;
use syntax::parse::{self, PResult};
use syntax::parse::parser::Parser as RustParser;
use syntax::parse::token::{self, DelimToken};
use syntax::ptr::P;

use super::render::{Escape, Renderer};

macro_rules! error {
    ($cx:expr, $sp:expr, $msg:expr) => ({
        $cx.span_err($sp, $msg);
        return Err(::syntax::diagnostic::FatalError);
    })
}
macro_rules! parse_error {
    ($self_:expr, $sp:expr, $msg:expr) => (error!($self_.render.cx, $sp, $msg))
}

macro_rules! dollar {
    () => (TtToken(_, token::Dollar))
}
macro_rules! pound {
    () => (TtToken(_, token::Pound))
}
macro_rules! dot {
    () => (TtToken(_, token::Dot))
}
macro_rules! eq {
    () => (TtToken(_, token::Eq))
}
macro_rules! not {
    () => (TtToken(_, token::Not))
}
macro_rules! question {
    () => (TtToken(_, token::Question))
}
macro_rules! semi {
    () => (TtToken(_, token::Semi))
}
macro_rules! minus {
    () => (TtToken(_, token::BinOp(token::Minus)))
}
macro_rules! slash {
    () => (TtToken(_, token::BinOp(token::Slash)))
}
macro_rules! literal {
    () => (TtToken(_, token::Literal(..)))
}
macro_rules! ident {
    ($x:pat) => (ident!(_, $x));
    ($sp:pat, $x:pat) => (TtToken($sp, token::Ident($x, token::IdentStyle::Plain)))
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
    try!(parser.markups());
    Ok(parser.into_render().into_expr(write.to_vec()))
}

pub fn split_comma<'a>(cx: &ExtCtxt, sp: Span, mac_name: &str, args: &'a [TokenTree])
    -> PResult<(&'a [TokenTree], &'a [TokenTree])>
{
    fn is_comma(t: &TokenTree) -> bool {
        match *t {
            TtToken(_, token::Comma) => true,
            _ => false,
        }
    }
    match args.iter().position(is_comma) {
        Some(i) => Ok((&args[..i], &args[1+i..])),
        None => error!(cx, sp, &format!("expected two arguments to `{}!`", mac_name)),
    }
}

struct Parser<'cx, 'i> {
    in_attr: bool,
    input: &'i [TokenTree],
    span: Span,
    render: Renderer<'cx>,
}

impl<'cx, 'i> Parser<'cx, 'i> {
    /// Finalizes the `Parser`, returning the `Renderer` underneath.
    fn into_render(self) -> Renderer<'cx> {
        let Parser { render, .. } = self;
        render
    }

    /// Consumes `n` items from the input.
    fn shift(&mut self, n: usize) {
        self.input = &self.input[n..];
    }

    /// Constructs a Rust AST parser from the given token tree.
    fn with_rust_parser<F, T>(&self, tts: Vec<TokenTree>, callback: F) -> T where
        F: FnOnce(&mut RustParser<'cx>) -> T
    {
        let mut parser = parse::tts_to_parser(self.render.cx.parse_sess, tts,
                                              self.render.cx.cfg.clone());
        let result = callback(&mut parser);
        // Make sure all tokens were consumed
        if parser.token != token::Eof {
            let token = parser.this_token_to_string();
            self.render.cx.span_err(parser.span,
                                    &format!("unexpected token: `{}`", token));
        }
        result
    }

    /// Parses and renders multiple blocks of markup.
    fn markups(&mut self) -> PResult<()> {
        loop {
            match self.input {
                [] => return Ok(()),
                [semi!(), ..] => self.shift(1),
                [_, ..] => try!(self.markup()),
            }
        }
    }

    /// Parses and renders a single block of markup.
    fn markup(&mut self) -> PResult<()> {
        match self.input {
            // Literal
            [minus!(), ref tt @ literal!(), ..] => {
                self.shift(2);
                try!(self.literal(tt, true));
            },
            [ref tt @ literal!(), ..] => {
                self.shift(1);
                try!(self.literal(tt, false))
            },
            // If
            [pound!(), ident!(sp, name), ..] if name.name == "if" => {
                self.shift(2);
                try!(self.if_expr(sp));
            },
            // For
            [pound!(), ident!(sp, name), ..] if name.name == "for" => {
                self.shift(2);
                try!(self.for_expr(sp));
            },
            // Call
            [pound!(), ident!(sp, name), ..] if name.name == "call" => {
                self.shift(2);
                let func = try!(self.splice(sp));
                self.render.emit_call(func);
            },
            // Splice
            [ref tt @ dollar!(), dollar!(), ..] => {
                self.shift(2);
                let expr = try!(self.splice(tt.get_span()));
                self.render.splice(expr, Escape::PassThru);
            },
            [ref tt @ dollar!(), ..] => {
                self.shift(1);
                let expr = try!(self.splice(tt.get_span()));
                self.render.splice(expr, Escape::Escape);
            },
            // Element
            [ident!(sp, _), ..] => {
                let name = try!(self.name());
                try!(self.element(sp, &name));
            },
            // Block
            [TtDelimited(_, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                {
                    // Parse the contents of the block, emitting the
                    // result inline
                    let mut i = &*d.tts;
                    mem::swap(&mut self.input, &mut i);
                    try!(self.markups());
                    mem::swap(&mut self.input, &mut i);
                }
            },
            // ???
            _ => {
                if let [ref tt, ..] = self.input {
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
        let lit = try!(self.with_rust_parser(vec![tt.clone()], RustParser::parse_lit));
        let s = try!(lit_to_string(self.render.cx, lit, minus));
        self.render.string(&s, Escape::Escape);
        Ok(())
    }

    /// Parses and renders an `#if` expression.
    ///
    /// The leading `#if` should already be consumed.
    fn if_expr(&mut self, sp: Span) -> PResult<()> {
        // Parse the initial if
        let mut if_cond = vec![];
        let if_body;
        loop { match self.input {
            [TtDelimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                if_body = try!(self.block(sp, &d.tts));
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                if_cond.push(tt.clone());
            },
            [] => parse_error!(self, sp, "expected body for this #if"),
        }}
        // Parse the (optional) else
        let else_body = match self.input {
            [pound!(), ident!(else_), ..] if else_.name == "else" => {
                self.shift(2);
                match self.input {
                    [ident!(sp, if_), ..] if if_.name == "if" => {
                        self.shift(1);
                        let else_body = {
                            // Parse an if expression, but capture the result
                            // rather than emitting it right away
                            let mut r = self.render.fork();
                            mem::swap(&mut self.render, &mut r);
                            try!(self.if_expr(sp));
                            mem::swap(&mut self.render, &mut r);
                            r.into_stmts()
                        };
                        Some(else_body)
                    },
                    [TtDelimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                        self.shift(1);
                        Some(try!(self.block(sp, &d.tts)))
                    },
                    _ => parse_error!(self, sp, "expected body for this #else"),
                }
            },
            _ => None,
        };
        self.render.emit_if(if_cond, if_body, else_body);
        Ok(())
    }

    /// Parses and renders a `#for` expression.
    ///
    /// The leading `#for` should already be consumed.
    fn for_expr(&mut self, sp: Span) -> PResult<()> {
        let mut pattern = vec![];
        loop { match self.input {
            [ident!(in_), ..] if in_.name == "in" => {
                self.shift(1);
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                pattern.push(tt.clone());
            },
            _ => parse_error!(self, sp, "invalid #for"),
        }}
        let pattern = self.with_rust_parser(pattern, RustParser::parse_pat);
        let mut iterable = vec![];
        let body;
        loop { match self.input {
            [TtDelimited(sp, ref d), ..] if d.delim == DelimToken::Brace => {
                self.shift(1);
                body = try!(self.block(sp, &d.tts));
                break;
            },
            [ref tt, ..] => {
                self.shift(1);
                iterable.push(tt.clone());
            },
            _ => parse_error!(self, sp, "invalid #for"),
        }}
        let iterable = self.with_rust_parser(iterable, RustParser::parse_expr);
        self.render.emit_for(pattern, iterable, body);
        Ok(())
    }

    /// Parses and renders a `$splice`.
    ///
    /// The leading `$` should already be consumed.
    fn splice(&mut self, sp: Span) -> PResult<P<Expr>> {
        // First, munch a single token tree
        let mut tts = match self.input {
            [ref tt, ..] => {
                self.shift(1);
                vec![tt.clone()]
            },
            [] => parse_error!(self, sp, "expected expression for this splice"),
        };
        loop { match self.input {
            // Munch attribute lookups e.g. `$person.address.street`
            [ref dot @ dot!(), ref ident @ ident!(_), ..] => {
                self.shift(2);
                tts.push(dot.clone());
                tts.push(ident.clone());
            },
            // Munch function calls `()` and indexing operations `[]`
            [TtDelimited(sp, ref d), ..] if d.delim != DelimToken::Brace => {
                self.shift(1);
                tts.push(TtDelimited(sp, d.clone()));
            },
            _ => break,
        }}
        Ok(self.with_rust_parser(tts, RustParser::parse_expr))
    }

    /// Parses and renders an element node.
    ///
    /// The element name should already be consumed.
    fn element(&mut self, sp: Span, name: &str) -> PResult<()> {
        if self.in_attr {
            parse_error!(self, sp, "unexpected element, you silly bumpkin");
        }
        self.render.element_open_start(name);
        try!(self.attrs());
        self.render.element_open_end();
        if let [slash!(), ..] = self.input {
            self.shift(1);
        } else {
            try!(self.markup());
            self.render.element_close(name);
        }
        Ok(())
    }

    /// Parses and renders the attributes of an element.
    fn attrs(&mut self) -> PResult<()> {
        loop {
            let old_input = self.input;
            let maybe_name = self.name();
            match (maybe_name, self.input) {
                (Ok(name), [eq!(), ..]) => {
                    // Non-empty attribute
                    self.shift(1);
                    self.render.attribute_start(&name);
                    {
                        // Parse a value under an attribute context
                        let mut in_attr = true;
                        mem::swap(&mut self.in_attr, &mut in_attr);
                        try!(self.markup());
                        mem::swap(&mut self.in_attr, &mut in_attr);
                    }
                    self.render.attribute_end();
                },
                (Ok(name), [question!(), ..]) => {
                    // Empty attribute
                    self.shift(1);
                    if let [ref tt @ eq!(), ..] = self.input {
                        // Toggle the attribute based on a boolean expression
                        self.shift(1);
                        let cond = try!(self.splice(tt.get_span()));
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
                _ => {
                    self.input = old_input;
                    break;
                },
        }}
        Ok(())
    }

    /// Parses a HTML element or attribute name.
    fn name(&mut self) -> PResult<String> {
        let mut s = match self.input {
            [ident!(name), ..] => {
                self.shift(1);
                String::from(&name.name.as_str() as &str)
            },
            _ => return Err(FatalError),
        };
        while let [minus!(), ident!(name), ..] = self.input {
            self.shift(2);
            s.push('-');
            s.push_str(&name.name.as_str());
        }
        Ok(s)
    }

    /// Parses the given token tree, returning a vector of statements.
    fn block(&mut self, sp: Span, tts: &[TokenTree]) -> PResult<Vec<P<Stmt>>> {
        let mut parse = Parser {
            in_attr: self.in_attr,
            input: tts,
            span: sp,
            render: self.render.fork(),
        };
        try!(parse.markups());
        Ok(parse.into_render().into_stmts())
    }
}

/// Converts a literal to a string.
fn lit_to_string(cx: &ExtCtxt, lit: Lit, minus: bool) -> PResult<String> {
    use syntax::ast::Lit_::*;
    let mut result = String::new();
    if minus {
        result.push('-');
    }
    match lit.node {
        LitStr(s, _) => result.push_str(&s),
        LitByteStr(..) | LitByte(..) => {
            error!(cx, lit.span, "cannot splice binary data");
        },
        LitChar(c) => result.push(c),
        LitInt(x, _) => result.push_str(&x.to_string()),
        LitFloat(s, _) | LitFloatUnsuffixed(s) => result.push_str(&s),
        LitBool(b) => result.push_str(if b { "true" } else { "false" }),
    };
    Ok(result)
}

/// If the expression is wrapped in parentheses, strip them off.
fn strip_outer_parens(expr: P<Expr>) -> P<Expr> {
    expr.and_then(|expr| match expr {
        Expr { node: ExprParen(inner), .. } => inner,
        expr => P(expr),
    })
}
