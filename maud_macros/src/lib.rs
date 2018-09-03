#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_non_items)]
#![feature(proc_macro_quote)]
#![feature(proc_macro_span)]
#![feature(tool_lints)]

#![doc(html_root_url = "https://docs.rs/maud_macros/0.18.1")]

// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate literalext;
#[macro_use] extern crate matches;
extern crate maud_htmlescape;
extern crate proc_macro;

mod ast;
mod generate;
mod parse;

use proc_macro::{Literal, Span, Ident, TokenStream, TokenTree};
use proc_macro::quote;

type ParseResult<T> = Result<T, ()>;

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    expand(input)
}

#[proc_macro]
pub fn html_debug(input: TokenStream) -> TokenStream {
    let expr = expand(input);
    println!("expansion:\n{}", expr);
    expr
}

fn expand(input: TokenStream) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", Span::def_site()));
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let size_hint = TokenTree::Literal(Literal::u64_unsuffixed(size_hint as u64));
    let markups = match parse::parse(input) {
        Ok(markups) => markups,
        Err(()) => Vec::new(),
    };
    let stmts = generate::generate(markups, output_ident.clone());
    quote!({
        extern crate maud;
        let mut $output_ident = String::with_capacity($size_hint);
        $stmts
        maud::PreEscaped($output_ident)
    })
}
