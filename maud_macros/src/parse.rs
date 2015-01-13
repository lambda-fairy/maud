use syntax::ast::{Expr, Lit, TokenTree, TtDelimited, TtToken};
use syntax::codemap::Span;
use syntax::ext::base::ExtCtxt;
use syntax::parse;
use syntax::parse::parser::Parser as RustParser;
use syntax::parse::token;
use syntax::ptr::P;

use super::render::{Escape, Renderer};

macro_rules! dollar {
    () => (TtToken(_, token::Dollar))
}
macro_rules! eq {
    () => (TtToken(_, token::Eq))
}
macro_rules! not {
    () => (TtToken(_, token::Not))
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

pub fn parse(cx: &mut ExtCtxt, input: &[TokenTree], sp: Span) -> P<Expr> {
    Renderer::with(cx, |render| {
        Parser {
            in_attr: false,
            input: input,
            span: sp,
            render: render,
        }.markups();
    })
}

struct Parser<'cx: 'r, 's: 'cx, 'i, 'r, 'o: 'r> {
    in_attr: bool,
    input: &'i [TokenTree],
    span: Span,
    render: &'r mut Renderer<'cx, 's, 'o>,
}

impl<'cx, 's, 'i, 'r, 'o> Parser<'cx, 's, 'i, 'r, 'o> {
    /// Consume `n` items from the input.
    fn shift(&mut self, n: usize) {
        self.input = self.input.slice_from(n);
    }

    /// Construct a Rust AST parser from the given token tree.
    fn new_rust_parser(&self, tts: Vec<TokenTree>) -> RustParser<'s> {
        parse::tts_to_parser(self.render.cx.parse_sess, tts, self.render.cx.cfg.clone())
    }

    fn markups(&mut self) {
        loop {
            match self.input {
                [] => return,
                [semi!(), ..] => self.shift(1),
                [_, ..] => if !self.markup() { return },
            }
        }
    }

    fn markup(&mut self) -> bool {
        match self.input {
            // Literal
            [minus!(), ref tt @ literal!(), ..] => {
                self.shift(2);
                self.literal(tt, true)
            },
            [ref tt @ literal!(), ..] => {
                self.shift(1);
                self.literal(tt, false)
            },
            // Splice
            [ref tt @ dollar!(), dollar!(), ..] => {
                self.shift(2);
                self.splice(Escape::PassThru, tt.get_span())
            },
            [ref tt @ dollar!(), ..] => {
                self.shift(1);
                self.splice(Escape::Escape, tt.get_span())
            },
            // Element
            [ident!(sp, name), ..] => {
                self.shift(1);
                self.element(name.as_str(), sp)
            },
            // Block
            [TtDelimited(_, ref d), ..] if d.delim == token::DelimToken::Brace => {
                self.shift(1);
                self.block(d.tts.as_slice())
            },
            // ???
            _ => {
                if let [ref tt, ..] = self.input {
                    self.render.cx.span_err(tt.get_span(), "invalid syntax");
                } else {
                    self.render.cx.span_err(self.span, "unexpected end of block");
                }
                return false;
            },
        }
        true
    }

    fn literal(&mut self, tt: &TokenTree, minus: bool) {
        let lit = self.new_rust_parser(vec![tt.clone()]).parse_lit();
        match lit_to_string(self.render.cx, lit, minus) {
            Some(s) => self.render.string(s.as_slice(), Escape::Escape),
            None => {},
        }
    }

    fn splice(&mut self, escape: Escape, sp: Span) {
        let tt = match self.input {
            [ref tt, ..] => {
                self.shift(1);
                self.new_rust_parser(vec![tt.clone()]).parse_expr()
            },
            _ => {
                self.render.cx.span_err(sp, "expected expression for this splice");
                return;
            },
        };
        self.render.splice(tt, escape);
    }

    fn element(&mut self, name: &str, sp: Span) {
        if self.in_attr {
            self.render.cx.span_err(sp, "unexpected element, you silly bumpkin");
            return;
        }
        self.render.element_open_start(name);
        self.attrs();
        self.render.element_open_end();
        if let [slash!(), ..] = self.input {
            self.shift(1);
        } else {
            self.markup();
            self.render.element_close(name);
        }
    }

    fn attrs(&mut self) {
        while let [ident!(name), eq!(), ..] = self.input {
            self.shift(2);
            if let [not!(), ..] = self.input {
                // Empty attribute
                self.shift(1);
                self.render.attribute_empty(name.as_str());
            } else {
                // Non-empty attribute
                self.render.attribute_start(name.as_str());
                {
                    // Parse a value under an attribute context
                    let old_in_attr = self.in_attr;
                    self.in_attr = true;
                    self.markup();
                    self.in_attr = old_in_attr;
                }
                self.render.attribute_end();
            }
        }
    }

    fn block(&mut self, tts: &[TokenTree]) {
        Parser {
            in_attr: self.in_attr,
            input: tts,
            span: self.span,
            render: self.render,
        }.markups();
    }
}

/// Convert a literal to a string.
fn lit_to_string(cx: &mut ExtCtxt, lit: Lit, minus: bool) -> Option<String> {
    use syntax::ast::Lit_::*;
    let mut result = String::new();
    if minus {
        result.push('-');
    }
    match lit.node {
        LitStr(s, _) => result.push_str(s.get()),
        LitBinary(..) | LitByte(..) => {
            cx.span_err(lit.span, "cannot splice binary data");
            return None;
        },
        LitChar(c) => result.push(c),
        LitInt(x, _) => result.push_str(x.to_string().as_slice()),
        LitFloat(s, _) | LitFloatUnsuffixed(s) => result.push_str(s.get()),
        LitBool(b) => result.push_str(if b { "true" } else { "false" }),
    };
    Some(result)
}
