#![crate_type = "dylib"]
#![feature(plugin_registrar, quote)]
#![feature(question_mark)]
#![feature(slice_patterns)]
#![feature(rustc_private)]

extern crate rustc_plugin;
extern crate syntax;
extern crate maud;

use rustc_plugin::Registry;
use syntax::ast::{Expr, TokenTree};
use syntax::codemap::{DUMMY_SP, Span};
use syntax::errors::FatalError;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::parse::token;
use syntax::print::pprust;
use syntax::ptr::P;

mod parse;
mod render;

pub type PResult<T> = Result<T, FatalError>;

fn html(cx: &mut ExtCtxt, sp: Span, mac_name: &str, args: &[TokenTree]) -> PResult<P<Expr>> {
    let (write, input) = parse::split_comma(cx, sp, mac_name, args)?;
    parse::parse(cx, sp, write, input)
}

fn html_utf8(cx: &mut ExtCtxt, sp: Span, mac_name: &str, args: &[TokenTree]) -> PResult<P<Expr>> {
    let (io_write, input) = parse::split_comma(cx, sp, mac_name, args)?;
    let io_write = io_write.to_vec();
    let fmt_write = token::gensym_ident("__maud_utf8_writer");
    let fmt_write = vec![
        TokenTree::Token(DUMMY_SP, token::Ident(fmt_write))];
    let expr = parse::parse(cx, sp, &fmt_write, input)?;
    Ok(quote_expr!(cx,
        match ::maud::Utf8Writer::new(&mut $io_write) {
            mut $fmt_write => {
                let _ = $expr;
                $fmt_write.into_result()
            }
        }))
}

macro_rules! generate_debug_wrappers {
    ($fn_name:ident $fn_debug_name:ident $mac_name:ident) => {
        fn $fn_name<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree])
            -> Box<MacResult + 'cx>
        {
            match $mac_name(cx, sp, stringify!($mac_name), args) {
                Ok(expr) => MacEager::expr(expr),
                Err(..) => DummyResult::expr(sp),
            }
        }

        fn $fn_debug_name<'cx>(cx: &'cx mut ExtCtxt, sp: Span, args: &[TokenTree])
            -> Box<MacResult + 'cx>
        {
            match $mac_name(cx, sp, concat!(stringify!($mac_name), "_debug"), args) {
                Ok(expr) => {
                    cx.span_warn(sp, &format!("expansion:\n{}",
                                              pprust::expr_to_string(&expr)));
                    MacEager::expr(expr)
                },
                Err(..) => DummyResult::expr(sp),
            }
        }
    }
}

generate_debug_wrappers!(expand_html expand_html_debug html);
generate_debug_wrappers!(expand_html_utf8 expand_html_utf8_debug html_utf8);

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("html", expand_html);
    reg.register_macro("html_debug", expand_html_debug);
    reg.register_macro("html_utf8", expand_html_utf8);
    reg.register_macro("html_utf8_debug", expand_html_utf8_debug);
}
