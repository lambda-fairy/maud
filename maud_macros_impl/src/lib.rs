#![doc(html_root_url = "https://docs.rs/maud_macros_impl/0.25.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

mod ast;
mod escape;
mod generate;
mod runtime;
mod parse;

use std::{io::{BufReader, BufRead}, fs::File};

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;

pub use parse::parse;
pub use runtime::format_str;

pub fn expand(input: TokenStream) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", Span::mixed_site()));
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let markups = parse::parse(input);

    let stmts = generate::generate(markups, output_ident.clone());
    quote!({
        extern crate alloc;
        extern crate maud;
        let mut #output_ident = alloc::string::String::with_capacity(#size_hint);
        #stmts
        maud::PreEscaped(#output_ident)
    })
}

// For the hot-reloadable version, maud will instead embed a tiny runtime
// that will render any markup-only changes. Any other changes will
// require a recompile. Of course, this is miles slower than the
// normal version, but it can be miles faster to iterate on.
pub fn expand_runtime(input: TokenStream) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", Span::mixed_site()));
    let markups = parse::parse(input);
    let stmts = runtime::generate(markups);
    quote!({
        extern crate alloc;
        extern crate maud;
        let mut #output_ident = String::new();
        let mut vars: ::std::collections::HashMap<&'static str, String> = ::std::collections::HashMap::new();

        let input = ::maud::gather_html_macro_invocations(file!(), line!());

        let markups = ::maud::parse(input.parse().unwrap());
        let format_str = ::maud::format_str(markups);

        #stmts

        let template = ::leon::Template::parse(&format_str).unwrap();

        maud::PreEscaped(template.render(&vars).unwrap())
    })
}

/// Grabs the inside of an html! {} invocation and returns it as a string
pub fn gather_html_macro_invocations(file_path: &'static str, start_column: u32) -> String {
    let buf_reader = BufReader::new(File::open(file_path).unwrap());

    let mut braces_diff = 0;

    let mut html_invocation = buf_reader
        .lines()
        .skip(start_column as usize)
        .take_while(|line| {
            let line = line.as_ref().unwrap();
            for c in line.chars() {
                if c == '{' {
                    braces_diff += 1;
                } else if c == '}' {
                    braces_diff -= 1;
                }
            }
            braces_diff != -1
        })
        .map(|line| line.unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    html_invocation
}
