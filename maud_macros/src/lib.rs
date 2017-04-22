#![crate_type = "dylib"]
#![feature(plugin_registrar, quote)]
#![feature(slice_patterns)]
#![feature(rustc_private)]
#![recursion_limit = "1000"]  // if_chain

#[macro_use]
extern crate if_chain;
#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;
extern crate maud;

use rustc_plugin::Registry;
use syntax::codemap::Span;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::print::pprust;
use syntax::tokenstream::TokenTree;

mod lints;
mod parse;
mod render;

type ParseResult<T> = Result<T, ()>;

fn expand_html<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'cx> {
    match parse::parse(cx, sp, args) {
        Ok(expr) => MacEager::expr(quote_expr!(cx, $expr)),
        Err(..) => DummyResult::expr(sp),
    }
}

fn expand_html_debug<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'cx> {
    match parse::parse(cx, sp, args) {
        Ok(expr) => {
            let expr = quote_expr!(cx, $expr);
            cx.span_warn(sp, &format!("expansion:\n{}",
                                      pprust::expr_to_string(&expr)));
            MacEager::expr(expr)
        },
        Err(..) => DummyResult::expr(sp),
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("html", expand_html);
    reg.register_macro("html_debug", expand_html_debug);
    lints::register_lints(reg);
}
