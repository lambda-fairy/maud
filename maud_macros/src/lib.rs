#![feature(proc_macro)]

#![doc(html_root_url = "https://docs.rs/maud_macros/0.17.1")]

extern crate literalext;
extern crate maud_htmlescape;
extern crate proc_macro;

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
