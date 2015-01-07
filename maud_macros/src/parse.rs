use syntax::ast::{Expr, Lit, TokenTree, TtDelimited, TtToken};
use syntax::ext::base::ExtCtxt;
use syntax::parse;
use syntax::parse::parser::Parser as RustParser;
use syntax::parse::token;
use syntax::ptr::P;

#[derive(Show)]
pub enum Markup {
    Element(String, Vec<(String, Value)>, Option<Box<Markup>>),
    Block(Vec<Markup>),
    Value(Value),
}

#[derive(Show)]
pub struct Value {
    pub value: Value_,
    pub escape: Escape,
}

#[derive(Show)]
pub enum Value_ {
    Literal(String),
    Splice(P<Expr>),
}

#[derive(Copy, PartialEq, Show)]
pub enum Escape {
    NoEscape,
    Escape,
}

macro_rules! some {
    ($e:expr) => (
        match $e {
            Some(x) => x,
            None => return None,
        }
    )
}

macro_rules! any {
    ($self_:expr;) => (None);
    ($self_:expr; $e:expr) => (any!($self_; $e,));
    ($self_:expr; $e:expr, $($es:expr),*) => ({
        let start_ptr = $self_.input.as_ptr();
        match $e {
            Some(x) => Some(x),
            None => {
                if $self_.input.as_ptr() == start_ptr {
                    // Parsing failed, but did not consume input.
                    // Keep going.
                    any!($self_; $($es),*)
                } else {
                    return None;
                }
            },
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

pub fn parse(cx: &mut ExtCtxt, input: &[TokenTree]) -> Option<Vec<Markup>> {
    Parser { cx: cx, input: input }.markups()
}

struct Parser<'cx, 's: 'cx, 'i> {
    cx: &'cx mut ExtCtxt<'s>,
    input: &'i [TokenTree],
}

impl<'cx, 's, 'i> Parser<'cx, 's, 'i> {
    /// Consume `n` items from the input.
    fn shift(&mut self, n: uint) {
        self.input = self.input.slice_from(n);
    }

    /// Construct a Rust AST parser from the given token tree.
    fn new_rust_parser(&self, tts: Vec<TokenTree>) -> RustParser<'s> {
        parse::tts_to_parser(self.cx.parse_sess, tts, self.cx.cfg.clone())
    }

    fn markups(&mut self) -> Option<Vec<Markup>> {
        let mut result = vec![];
        loop {
            match self.input {
                [] => return Some(result),
                [semi!(), ..] => self.shift(1),
                [ref tt, ..] => {
                    match self.markup() {
                        Some(markup) => result.push(markup),
                        None => {
                            self.cx.span_err(tt.get_span(), "invalid syntax");
                            return None;
                        },
                    }
                }
            }
        }
    }

    fn markup(&mut self) -> Option<Markup> {
        any!(self;
            self.value().map(Markup::Value),
            self.block(),
            self.element())
    }

    fn value(&mut self) -> Option<Value> {
        any!(self;
            self.literal(),
            self.splice())
    }

    fn literal(&mut self) -> Option<Value> {
        let (tt, minus) = match self.input {
            [minus!(), ref tt @ literal!(), ..] => {
                self.shift(2);
                (tt, true)
            },
            [ref tt @ literal!(), ..] => {
                self.shift(1);
                (tt, false)
            },
            _ => return None,
        };
        let lit = self.new_rust_parser(vec![tt.clone()]).parse_lit();
        lit_to_string(self.cx, lit, minus)
            .map(|s| Value {
                value: Value_::Literal(s),
                escape: Escape::Escape,
            })
    }

    fn splice(&mut self) -> Option<Value> {
        let (escape, sp) = match self.input {
            [ref tt @ dollar!(), dollar!(), ..] => {
                self.shift(2);
                (Escape::NoEscape, tt.get_span())
            },
            [ref tt @ dollar!(), ..] => {
                self.shift(1);
                (Escape::Escape, tt.get_span())
            },
            _ => return None,
        };
        let tt = match self.input {
            [ref tt, ..] => {
                self.shift(1);
                self.new_rust_parser(vec![tt.clone()]).parse_expr()
            },
            _ => {
                self.cx.span_err(sp, "expected expression for this splice");
                return None;
            },
        };
        Some(Value {
            value: Value_::Splice(tt),
            escape: escape,
        })
    }

    fn element(&mut self) -> Option<Markup> {
        let name = match self.input {
            [ident!(name), ..] => {
                self.shift(1);
                name.as_str().to_string()
            },
            _ => return None,
        };
        let attrs = some!(self.attrs());
        let body = any!(self; self.markup());
        Some(Markup::Element(name, attrs, body.map(|body| box body)))
    }

    fn attrs(&mut self) -> Option<Vec<(String, Value)>> {
        let mut attrs = vec![];
        while let [ident!(name), eq!(), ..] = self.input {
            self.shift(2);
            let name = name.as_str().to_string();
            let value = some!(self.value());
            attrs.push((name, value));
        }
        Some(attrs)
    }

    fn block(&mut self) -> Option<Markup> {
        match self.input {
            [TtDelimited(_, ref d), ..] if d.delim == token::DelimToken::Brace => {
                self.shift(1);
                Parser { cx: self.cx, input: d.tts[] }.markups()
                    .map(Markup::Block)
            },
            _ => None,
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
        LitInt(x, _) => result.push_str(x.to_string()[]),
        LitFloat(s, _) | LitFloatUnsuffixed(s) => result.push_str(s.get()),
        LitBool(b) => result.push_str(if b { "true" } else { "false" }),
    };
    Some(result)
}
