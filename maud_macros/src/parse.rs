use syntax::ast::{Expr, Lit, TokenTree, TtToken};
use syntax::ext::base::ExtCtxt;
use syntax::parse;
use syntax::parse::token;
use syntax::ptr::P;

#[deriving(Show)]
pub enum Markup {
    Element(Vec<(String, Value)>, Vec<Markup>),
    Value(Value),
}

#[deriving(Show)]
pub struct Value {
    pub value: Value_,
    pub escape: Escape,
}

impl Value {
    pub fn escape(value: Value_) -> Value {
        Value {
            value: value,
            escape: Escape::Escape,
        }
    }

    pub fn no_escape(value: Value_) -> Value {
        Value {
            value: value,
            escape: Escape::NoEscape,
        }
    }
}

#[deriving(Show)]
pub enum Value_ {
    Literal(String),
    Splice(P<Expr>),
}

#[deriving(Copy, PartialEq, Show)]
pub enum Escape {
    NoEscape,
    Escape,
}

pub fn parse(cx: &mut ExtCtxt, mut args: &[TokenTree]) -> Option<Vec<Markup>> {
    macro_rules! minus {
        () => (TtToken(_, token::BinOp(token::Minus)))
    }
    macro_rules! literal {
        () => (TtToken(_, token::Literal(..)))
    }

    let mut result = vec![];
    loop {
        match match args {
            [minus!(), ref tt @ literal!(), ..] => {
                args.shift(2);
                parse_literal(cx, tt, true)
            },
            [ref tt @ literal!(), ..] => {
                args.shift(1);
                parse_literal(cx, tt, false)
            },
            _ => None,
        } {
            Some(x) => result.push(x),
            None => break,
        }
    }
    match args {
        [] => Some(result),
        [ref tt, ..] => {
            cx.span_err(tt.get_span(), "invalid syntax");
            None
        }
    }
}

fn parse_literal(cx: &mut ExtCtxt, tt: &TokenTree, minus: bool) -> Option<Markup> {
    let mut parser = parse::tts_to_parser(cx.parse_sess, vec![tt.clone()], cx.cfg.clone());
    let lit = parser.parse_lit();
    lit_to_string(cx, lit, minus)
        .map(|s| Markup::Value(Value::escape(Value_::Literal(s))))
}

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
        LitInt(x, _) => result.push_str(&*x.to_string()),
        LitFloat(s, _) | LitFloatUnsuffixed(s) => result.push_str(s.get()),
        LitBool(b) => result.push_str(if b { "true" } else { "false" }),
    };
    Some(result)
}

trait Shift {
    fn shift(&mut self, n: uint);
}

impl<'a, T> Shift for &'a [T] {
    fn shift(&mut self, n: uint) {
        *self = self.slice_from(n);
    }
}
