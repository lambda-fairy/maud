#![feature(proc_macro_def_site)]
#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_hygiene)]
#![feature(proc_macro_quote)]
#![feature(proc_macro_span)]

#![doc(html_root_url = "https://docs.rs/maud_macros/0.21.0")]

// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod generate;
mod parse;

use proc_macro2::{Literal, Ident, TokenStream, TokenTree};
// use proc_macro::quote;
use quote::quote;

type ParseResult<T> = Result<T, ()>;

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

#[proc_macro]
pub fn html_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expr = expand(input.into());
    println!("expansion:\n{}", expr);
    expr.into()
}

fn expand(input: TokenStream) -> TokenStream {
    // using proc_macro::Span here allows us to work around the limitation set by proc_macro2 on Span
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", proc_macro::Span::mixed_site().into()));
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let size_hint = TokenTree::Literal(Literal::u64_unsuffixed(size_hint as u64));
    let markups = match parse::parse(proc_macro::TokenStream::from(input)) {
        Ok(markups) => markups,
        Err(()) => Vec::new(),
    };
    let stmts = generate::generate(markups, output_ident.clone());
    TokenStream::from(
        quote!({
            extern crate maud;
            let mut #output_ident = ::std::string::String::with_capacity(#size_hint);
            #stmts
            maud::PreEscaped(#output_ident)
        })
    )
}
