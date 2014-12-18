#![crate_type = "dylib"]
#![feature(globs, plugin_registrar, quote, macro_rules)]

extern crate syntax;
extern crate rustc;

use syntax::ast::{Ident, TokenTree};
use syntax::codemap::Span;
use syntax::ext::base::{DummyResult, ExtCtxt, IdentTT, MacItems, MacResult};
use syntax::parse::token;
use rustc::plugin::Registry;

mod parse;
mod render;

fn expand_html<'cx>(cx: &'cx mut ExtCtxt, sp: Span, ident: Ident, args: Vec<TokenTree>) -> Box<MacResult + 'cx> {
    match parse::parse(cx, &*args) {
        Some(markups) => {
            let item = render::render(cx, ident, &*markups);
            MacItems::new(item.into_iter())
        },
        None => DummyResult::any(sp),
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern("html"), IdentTT(box expand_html, None));
}
