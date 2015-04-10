#![crate_type = "dylib"]
#![feature(plugin_registrar, quote)]
#![feature(slice_patterns)]
#![feature(rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate maud;

use syntax::ast::TokenTree;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacEager, MacResult};
use syntax::print::pprust;
use rustc::plugin::Registry;

mod parse;
mod render;

fn expand_html<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'cx> {
    let expr = parse::parse(cx, args, sp);
    if cfg!(feature = "print-expansion") {
        println!("{}", pprust::expr_to_string(&expr));
    }
    MacEager::expr(expr)
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("html", expand_html);
}
