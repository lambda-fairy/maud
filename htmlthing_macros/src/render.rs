use syntax::ast::{Ident, Item, Stmt};
use syntax::ext::base::ExtCtxt;
use syntax::parse::token;
use syntax::ptr::P;

use super::parse::Markup;

pub fn render(cx: &mut ExtCtxt, ident: Ident, markups: &[Markup]) -> Option<P<Item>> {
    let w = Ident::new(token::intern("w"));
    let mut stmts = vec![];
    for markup in markups.iter() {
        render_markup(cx, markup, w, &mut stmts);
    }
    quote_item!(cx,
        fn $ident<W: ::std::io::Writer>($w: &mut W) -> ::std::io::IoResult<()> {
            $stmts;
            Ok(())
        }
    )
}

fn render_markup(cx: &mut ExtCtxt, markup: &Markup, w: Ident, out: &mut Vec<P<Stmt>>) {
    use super::parse::Markup::*;
    use super::parse::Value::*;
    match *markup {
        Empty => {},
        Element(..) => unimplemented!(),
        Value(Literal(ref s)) => {
            out.push(quote_stmt!(cx, try!($w.write_str($s))));
        },
        Value(Splice(_)) => unimplemented!(),
    }
}
