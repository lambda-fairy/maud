#![doc(html_root_url = "https://docs.rs/maud_macros/0.25.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod escape;
mod generate;
mod parse;

use proc_macro2::{TokenStream, TokenTree};
use proc_macro_error::{proc_macro_error, abort};
use quote::quote;

#[proc_macro]
#[proc_macro_error]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

#[proc_macro]
#[proc_macro_error]
pub fn html_to(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_to(input.into()).into()
}

fn expand(input: TokenStream) -> TokenStream {
    quote!({
        extern crate alloc;
        extern crate maud;
        let mut __maud_output = alloc::string::String::new();
        maud::html_to!(__maud_output, #input);
        maud::PreEscaped(__maud_output)
    })
}

fn expand_to(input: TokenStream) -> TokenStream {
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    // TODO: Better place for error handling? 
    let (output_ident, markups) = match parse::parse(input.clone()) {
        (Some(ident), markups) => (ident, markups),
        _ => abort!(
            input,
            "expected mutable String buffer"
        )
    };
    
    let stmts = generate::generate(markups, TokenTree::Ident(output_ident.clone()));
    quote!({
        extern crate maud;
        #output_ident.reserve(#size_hint);
        #stmts
    })
}