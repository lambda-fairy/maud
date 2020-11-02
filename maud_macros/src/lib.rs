#![doc(html_root_url = "https://docs.rs/maud_macros/0.22.1")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod generate;
mod parse;

use proc_macro2::{Ident, TokenStream, TokenTree};
use proc_macro_error::proc_macro_error;
use quote::quote;

#[proc_macro]
#[proc_macro_error]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

#[proc_macro]
#[proc_macro_error]
pub fn html_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expr = expand(input.into());
    println!("expansion:\n{}", expr);
    expr.into()
}

fn expand(input: TokenStream) -> TokenStream {
    // TODO: call `proc_macro2::Span::mixed_site()` directly when Rust 1.45 is stable
    let output_ident = TokenTree::Ident(Ident::new(
        "__maud_output",
        proc_macro::Span::mixed_site().into(),
    ));
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let markups = parse::parse(input);
    let stmts = generate::generate(markups, output_ident.clone());
    quote!({
        extern crate maud;
        let mut #output_ident = ::std::string::String::with_capacity(#size_hint);
        #stmts
        maud::PreEscaped(#output_ident)
    })
}
