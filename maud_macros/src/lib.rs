#![crate_type = "dylib"]
#![feature(plugin_registrar, quote)]
#![feature(slice_patterns)]
#![feature(rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate maud;

use syntax::ast::{Expr, TokenTree, TtToken};
use syntax::codemap::{DUMMY_SP, Span};
use syntax::ext::base::{ExtCtxt, MacEager, MacResult};
use syntax::parse::token;
use syntax::print::pprust;
use syntax::ptr::P;
use rustc::plugin::Registry;

mod parse;
mod render;

fn _expand_html(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> P<Expr> {
    let (write, input) = parse::split_comma(cx, sp, args);
    parse::parse(cx, sp, write, input)
}

fn _expand_html_utf8(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> P<Expr> {
    let (io_write, input) = parse::split_comma(cx, sp, args);
    let io_write = io_write.to_vec();
    let fmt_write = token::gensym_ident("__maud_utf8_writer");
    let fmt_write = vec![
        TtToken(DUMMY_SP, token::Ident(fmt_write, token::IdentStyle::Plain))];
    let expr = parse::parse(cx, sp, &fmt_write, input);
    quote_expr!(cx,
        match ::maud::Utf8Writer::new(&mut $io_write) {
            mut $fmt_write => {
                let _ = $expr;
                $fmt_write.into_result()
            }
        })
}

macro_rules! generate_debug_wrappers {
    ($name:ident $debug_name:ident $inner_fn:ident) => {
        fn $name<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree])
            -> Box<MacResult + 'cx>
        {
            let expr = $inner_fn(cx, sp, args);
            MacEager::expr(expr)
        }

        fn $debug_name<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree])
            -> Box<MacResult + 'cx>
        {
            let expr = $inner_fn(cx, sp, args);
            cx.span_note(sp, &format!("expansion:\n{}",
                                      pprust::expr_to_string(&expr)));
            MacEager::expr(expr)
        }
    }
}

generate_debug_wrappers!(expand_html expand_html_debug _expand_html);
generate_debug_wrappers!(expand_html_utf8 expand_html_utf8_debug _expand_html_utf8);

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("html", expand_html);
    reg.register_macro("html_debug", expand_html_debug);
    reg.register_macro("html_utf8", expand_html_utf8);
    reg.register_macro("html_utf8_debug", expand_html_utf8_debug);
}
