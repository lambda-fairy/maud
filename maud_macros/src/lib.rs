#![feature(proc_macro)]
#![recursion_limit = "1000"]  // if_chain

#![doc(html_root_url = "https://docs.rs/maud_macros/0.16.3")]

#[macro_use]
extern crate if_chain;
extern crate literalext;
extern crate proc_macro;

// TODO move lints into their own `maud_lints` crate
// mod lints;
mod parse;
mod render;

use proc_macro::TokenStream;

type ParseResult<T> = Result<T, String>;

#[proc_macro]
pub fn html(args: TokenStream) -> TokenStream {
    match parse::parse(args) {
        Ok(expr) => expr,
        Err(e) => panic!(e),
    }
}

#[proc_macro]
pub fn html_debug(args: TokenStream) -> TokenStream {
    match parse::parse(args) {
        Ok(expr) => {
            println!("expansion:\n{}", expr);
            expr
        },
        Err(e) => panic!(e),
    }
}

/*
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("html", expand_html);
    reg.register_macro("html_debug", expand_html_debug);
    lints::register_lints(reg);
}
*/
