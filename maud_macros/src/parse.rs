use syntax::ast::{Expr, Lit, TokenTree, TtDelimited, TtToken};
use syntax::codemap::Span;
use syntax::ext::base::ExtCtxt;
use syntax::parse;
use syntax::parse::parser::Parser as RustParser;
use syntax::parse::token;
use syntax::ptr::P;

use super::render::{Escape, Renderer};

macro_rules! guard {
    ($e:expr) => (if !$e { return false; })
}

macro_rules! dollar {
    () => (TtToken(_, token::Dollar))
}
macro_rules! eq {
    () => (TtToken(_, token::Eq))
}
macro_rules! semi {
    () => (TtToken(_, token::Semi))
}
macro_rules! minus {
    () => (TtToken(_, token::BinOp(token::Minus)))
}
macro_rules! literal {
    () => (TtToken(_, token::Literal(..)))
}
macro_rules! ident {
    ($x:pat) => (ident!(_, $x));
    ($sp:pat, $x:pat) => (TtToken($sp, token::Ident($x, token::IdentStyle::Plain)))
}

pub fn parse(cx: &mut ExtCtxt, input: &[TokenTree]) -> Option<P<Expr>> {
    let mut success = true;
    let expr = Renderer::with(cx, |render| {
        let mut parser = Parser {
            in_attr: false,
            input: input,
            render: render,
        };
        success = parser.markups();
    });
    if success {
        Some(expr)
    } else {
        None
    }
}

struct Parser<'cx: 'r, 's: 'cx, 'i, 'r, 'o: 'r> {
    in_attr: bool,
    input: &'i [TokenTree],
    render: &'r mut Renderer<'cx, 's, 'o>,
}

impl<'cx: 'r, 's: 'cx, 'i, 'r, 'o: 'r> Parser<'cx, 's, 'i, 'r, 'o> {
    /// Consume `n` items from the input.
    fn shift(&mut self, n: usize) {
        self.input = self.input.slice_from(n);
    }

    /// Construct a Rust AST parser from the given token tree.
    fn new_rust_parser(&self, tts: Vec<TokenTree>) -> RustParser<'s> {
        parse::tts_to_parser(self.render.cx.parse_sess, tts, self.render.cx.cfg.clone())
    }

    fn markups(&mut self) -> bool {
        loop {
            match self.input {
                [] => return true,
                [semi!(), ..] => self.shift(1),
                [_, ..] => guard!(self.markup()),
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
                }
                false
            },
        }
    }

    fn literal(&mut self, tt: &TokenTree, minus: bool) -> bool {
        let lit = self.new_rust_parser(vec![tt.clone()]).parse_lit();
        match lit_to_string(self.render.cx, lit, minus) {
            Some(s) => self.render.string(s.as_slice(), Escape::Escape),
            None => return false,
        }
        true
    }

    fn splice(&mut self, escape: Escape, sp: Span) -> bool {
        let tt = match self.input {
            [ref tt, ..] => {
                self.shift(1);
                self.new_rust_parser(vec![tt.clone()]).parse_expr()
            },
            _ => {
                self.render.cx.span_err(sp, "expected expression for this splice");
                return false;
            },
        };
        self.render.splice(tt, escape);
        true
    }

    fn element(&mut self, name: &str, sp: Span) -> bool {
        if self.in_attr {
            self.render.cx.span_err(sp, "unexpected element, you silly bumpkin");
            return false;
        }
        self.render.element_open_start(name);
        guard!(self.attrs());
        self.render.element_open_end();
        guard!(self.markup());
        self.render.element_close(name);
        true
    }

    fn attrs(&mut self) -> bool {
        while let [ident!(name), eq!(), ..] = self.input {
            self.shift(2);
            self.render.attribute_start(name.as_str());
            {
                let old_in_attr = self.in_attr;
                self.in_attr = true;
                guard!(self.markup());
                self.in_attr = old_in_attr;
            }
            self.render.attribute_end();
        }
        true
    }

    fn block(&mut self, tts: &[TokenTree]) -> bool {
        Parser {
            in_attr: self.in_attr,
            input: tts,
            render: self.render,
        }.markups()
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
