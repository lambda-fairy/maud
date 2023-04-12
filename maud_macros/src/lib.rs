#![doc(html_root_url = "https://docs.rs/maud_macros/0.24.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod escape;
mod generate;
mod parse;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
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
        maud::html_to!(__maud_output #input);
        maud::PreEscaped(__maud_output)
    })
}

fn expand_to(input: TokenStream) -> TokenStream {
    // TODO: Better/more beatiful way to get ident? 
    let mut iter = input.clone().into_iter();

    // TODO: Better place for error handling?
    let output_ident = match iter.next() {
        Some(ident @ TokenTree::Ident(_)) => ident,
        Some(token) => abort!(
            token,
            "expected mutable String buffer",
        ),
        None => abort!(
            input,
            "expected mutable String buffer"
        ),
    };
    
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let markups = parse::parse(iter.collect());
    let stmts = generate::generate(markups, output_ident.clone());
    quote!({
        extern crate maud;
        #output_ident.reserve(#size_hint);
        #stmts
    })
}