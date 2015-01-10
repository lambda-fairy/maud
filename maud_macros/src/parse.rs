use syntax::ast::{Expr, Lit, TokenTree, TtDelimited, TtToken};
use syntax::ext::base::ExtCtxt;
use syntax::parse;
use syntax::parse::parser::Parser as RustParser;
use syntax::parse::token;
use syntax::ptr::P;

use super::render::Renderer;

#[derive(Copy, PartialEq, Show)]
pub enum Escape {
    None,
    Attr,
    Body,
}

macro_rules! guard {
    ($e:expr) => (if !$e { return false; })
}

macro_rules! branch {
    ($self_:expr;) => (return false);
    ($self_:expr; $e:expr) => (branch!($self_; $e,));
    ($self_:expr; $e:expr, $($es:expr),*) => ({
        let start_ptr = $self_.input.as_ptr();
        if $e {
            true
        } else {
            if $self_.input.as_ptr() == start_ptr {
                // Parsing failed, but did not consume input.
                // Keep going.
                branch!($self_; $($es),*)
            } else {
                return false;
            }
        }
    })
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
    ($x:pat) => (TtToken(_, token::Ident($x, token::IdentStyle::Plain)))
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

    fn choose_escape(&self) -> Escape {
        if self.in_attr {
            Escape::Attr
        } else {
            Escape::Body
        }
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
                [ref tt, ..] => {
                    if !self.markup() {
                        self.render.cx.span_err(tt.get_span(), "invalid syntax");
                        return false;
                    }
                }
            }
        }
    }

    fn markup(&mut self) -> bool {
        branch!(self;
            self.literal(),
            self.splice(),
            self.block(),
            !self.in_attr && self.element())
    }

    fn literal(&mut self) -> bool {
        let (tt, minus) = match self.input {
            [minus!(), ref tt @ literal!(), ..] => {
                self.shift(2);
                (tt, true)
            },
            [ref tt @ literal!(), ..] => {
                self.shift(1);
                (tt, false)
            },
            _ => return false,
        };
        let lit = self.new_rust_parser(vec![tt.clone()]).parse_lit();
        match lit_to_string(self.render.cx, lit, minus) {
            Some(s) => {
                let escape = self.choose_escape();
                self.render.string(s.as_slice(), escape);
            },
            None => return false,
        }
        true
    }

    fn splice(&mut self) -> bool {
        let (escape, sp) = match self.input {
            [ref tt @ dollar!(), dollar!(), ..] => {
                self.shift(2);
                (Escape::None, tt.get_span())
            },
            [ref tt @ dollar!(), ..] => {
                self.shift(1);
                (self.choose_escape(), tt.get_span())
            },
            _ => return false,
        };
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

    fn element(&mut self) -> bool {
        let name = match self.input {
            [ident!(name), ..] => {
                self.shift(1);
                name.as_str().to_string()
            },
            _ => return false,
        };
        let name = name.as_slice();
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

    fn block(&mut self) -> bool {
        match self.input {
            [TtDelimited(_, ref d), ..] if d.delim == token::DelimToken::Brace => {
                self.shift(1);
                Parser {
                    in_attr: self.in_attr,
                    input: d.tts.as_slice(),
                    render: self.render,
                }.markups()
            },
            _ => false,
        }
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
