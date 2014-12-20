use syntax::ast::{Expr, Ident, Stmt};
use syntax::ext::base::ExtCtxt;
use syntax::parse::token;
use syntax::ptr::P;

use super::parse::{Markup, Value};
use maud::escape;

pub fn render(cx: &mut ExtCtxt, markups: &[Markup]) -> P<Expr> {
    let w = Ident::new(token::intern("w"));
    let mut stmts = vec![];
    for markup in markups.iter() {
        render_markup(cx, markup, w, &mut stmts);
    }
    quote_expr!(cx, |&: $w: &mut ::std::io::Writer| -> ::std::io::IoResult<()> {
        $stmts
        Ok(())
    })
}

fn render_markup(cx: &mut ExtCtxt, markup: &Markup, w: Ident, out: &mut Vec<P<Stmt>>) {
    use super::parse::Markup::*;
    match *markup {
        Element(..) => unimplemented!(),
        Value(ref value) => {
            let expr = render_value(cx, value, w, false);
            out.push(quote_stmt!(cx, $expr));
        },
    }
}

fn render_value(cx: &mut ExtCtxt, value: &Value, w: Ident, is_attr: bool) -> P<Expr> {
    use super::parse::Escape::*;
    use super::parse::Value_::*;
    let &Value { ref value, escape } = value;
    match *value {
        Literal(ref s) => {
            let s = match escape {
                NoEscape => (&**s).into_cow(),
                Escape => if is_attr {
                    escape::attribute(&**s).into_cow()
                } else {
                    escape::non_attribute(&**s).into_cow()
                },
            };
            quote_expr!(cx, {
                try!($w.write_str($s))
            })
        },
        Splice(ref expr) => match escape {
            NoEscape => quote_expr!(cx, {
                try!(write!($w, "{}", $expr));
            }),
            Escape => quote_expr!(cx, {
                let s = $expr.to_string();
                for c in s.chars() {
                    try!(if $is_attr {
                            ::maud::rt::escape_attribute(c, $w)
                        } else {
                            ::maud::rt::escape_non_attribute(c, $w)
                        });
                }
            }),
        },
    }
}
