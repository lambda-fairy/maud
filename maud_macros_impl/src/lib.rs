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

pub use parse::parse_at_runtime;
pub use runtime::format_str;

use crate::ast::Markup;

pub fn expand(input: TokenStream) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", Span::mixed_site()));
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let markups = parse::parse(input);

    expand_from_parsed(markups, output_ident, size_hint)
}

fn expand_from_parsed(markups: Vec<Markup>, output_ident: TokenTree, size_hint: usize) -> TokenStream {
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
    let markups = parse::parse(input.clone());
    let stmts = runtime::generate(markups);
    quote!({
        extern crate alloc;
        extern crate maud;
        let mut #output_ident = String::new();
        let mut vars: ::std::collections::HashMap<&'static str, String> = ::std::collections::HashMap::new();

        let input = ::maud::macro_private::gather_html_macro_invocations(file!(), line!());

        let res = ::std::panic::catch_unwind(|| {
            ::maud::macro_private::parse_at_runtime(input.parse().unwrap())
        });

        if let Err(e) = res {
            if let Some(s) = e
                // Try to convert it to a String, then turn that into a str
                .downcast_ref::<String>()
                .map(String::as_str)
                // If that fails, try to turn it into a &'static str
                .or_else(|| e.downcast_ref::<&'static str>().map(::std::ops::Deref::deref))
            {
                ::maud::macro_private::render_runtime_error(s)
            } else {
                ::maud::macro_private::render_runtime_error("unknown panic")
            }
        } else {
            let markups = ::maud::macro_private::parse_at_runtime(input.parse().unwrap());
            let format_str = ::maud::macro_private::format_str(markups);

            #stmts

            // cannot use return here, and block labels come with strings attached (cant nest them
            // without compiler warnings)
            match ::maud::macro_private::leon::Template::parse(&format_str) {
                Ok(template) => {
                    match template.render(&vars) {
                        Ok(template) => maud::PreEscaped(template),
                        Err(e) => ::maud::macro_private::render_runtime_error(&e.to_string())
                    }
                },
                Err(e) => ::maud::macro_private::render_runtime_error(&e.to_string())
            }
        }
    })
}

/// Grabs the inside of an html! {} invocation and returns it as a string
pub fn gather_html_macro_invocations(file_path: &'static str, start_column: u32) -> String {
    let buf_reader = BufReader::new(File::open(file_path).unwrap());

    let mut braces_diff = 0;

    let html_invocation = buf_reader
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
