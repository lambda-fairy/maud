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

fn expand_html_common<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree],
                           debug: bool) -> Box<MacResult + 'cx> {
    let expr = parse::parse(cx, args, sp);
    if debug {
        cx.span_note(sp, &format!("expansion:\n{}",
                                  pprust::expr_to_string(&expr)));
    }
    MacEager::expr(expr)
}

fn expand_html<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'cx> {
    expand_html_common(cx, sp, args, false)
}

fn expand_html_debug<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'cx> {
    expand_html_common(cx, sp, args, true)
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("html", expand_html);
    reg.register_macro("html_debug", expand_html_debug);
}
