#![crate_type = "dylib"]
#![feature(globs, plugin_registrar, quote, macro_rules)]

extern crate syntax;
extern crate rustc;
extern crate maud;

use syntax::ast::TokenTree;
use syntax::codemap::Span;
use syntax::ext::base::{DummyResult, ExtCtxt, MacExpr, MacResult};
use rustc::plugin::Registry;

mod parse;
mod render;

fn expand_html<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'cx> {
    match parse::parse(cx, &*args) {
        Some(markups) => {
            let expr = render::render(cx, &*markups);
            MacExpr::new(expr)
        },
        None => DummyResult::any(sp),
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("html", expand_html);
}
