#![doc(html_root_url = "https://docs.rs/maud_macros_impl/0.25.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

mod ast;
mod escape;
mod generate;
#[cfg(feature = "hotreload")]
mod runtime;
mod parse;

use std::{io::{BufReader, BufRead}, fs::File, collections::HashMap};

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;

use crate::{ast::Markup, parse::parse_at_runtime};

#[cfg(feature = "hotreload")]
use crate::runtime::format_str;

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
        ::maud::PreEscaped(#output_ident)
    })
}

// For the hot-reloadable version, maud will instead embed a tiny runtime
// that will render any markup-only changes. Any other changes will
// require a recompile. Of course, this is miles slower than the
// normal version, but it can be miles faster to iterate on.
#[cfg(feature = "hotreload")]
pub fn expand_runtime(input: TokenStream) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", Span::mixed_site()));
    let markups = parse::parse(input.clone());
    let stmts = runtime::generate(markups);
    quote!({
        extern crate alloc;
        extern crate maud;
        let mut #output_ident = String::new();
        let file_info = file!();
        let line_info = line!();

        let input = ::maud::macro_private::gather_html_macro_invocations(file_info, line_info);
        let mut vars: ::std::collections::HashMap<&'static str, String> = ::std::collections::HashMap::new();
        #stmts;

        match ::maud::macro_private::expand_runtime_main(
            vars,
            input.as_deref(),
            file_info,
            line_info,
        ) {
            Ok(x) => ::maud::PreEscaped(x),
            Err(e) => ::maud::macro_private::render_runtime_error(&input.unwrap_or_default(), &e),
        }
    })
}

#[cfg(feature = "hotreload")]
pub fn expand_runtime_main(vars: HashMap<&'static str, String>, input: Option<&str>, file_info: &str, line_info: u32) -> Result<String, String> {
    if let Some(input) = input {
        let res = ::std::panic::catch_unwind(|| {
            parse_at_runtime(input.parse().unwrap())
        });

        if let Err(e) = res {
            if let Some(s) = e
                // Try to convert it to a String, then turn that into a str
                .downcast_ref::<String>()
                .map(String::as_str)
                // If that fails, try to turn it into a &'static str
                .or_else(|| e.downcast_ref::<&'static str>().map(::std::ops::Deref::deref))
            {
                return Err(s.to_string());
            } else {
                return Err("unknown panic".to_owned());
            }
        } else {
            let markups = parse_at_runtime(input.parse().unwrap());
            let format_str = format_str(markups);

            // cannot use return here, and block labels come with strings attached (cant nest them
            // without compiler warnings)
            match leon::Template::parse(&format_str) {
                Ok(template) => {
                    match template.render(&vars) {
                        Ok(template) => Ok(template),
                        Err(e) => Err(e.to_string())
                    }
                },
                Err(e) => Err(e.to_string())
            }
        }
    } else {
        Err(format!("can't find template source at {}:{}, please recompile", file_info, line_info))
    }
}

/// Grabs the inside of an html! {} invocation and returns it as a string
pub fn gather_html_macro_invocations(file_path: &'static str, start_line: u32) -> Option<String> {
    let buf_reader = BufReader::new(File::open(file_path).unwrap());

    let mut braces_diff = 0;

    let html_invocation = buf_reader
        .lines()
        .skip(start_line as usize - 1)
        .map(|line| line.unwrap())
        // scan for beginning of the macro. start_line may point to it directly, but we want to
        // handle code flowing slightly downward.
        .skip_while(|line| !line.contains("html!"))
        .skip(1)
        .take_while(|line| {
            for c in line.chars() {
                if c == '{' {
                    braces_diff += 1;
                } else if c == '}' {
                    braces_diff -= 1;
                }
            }
            braces_diff != -1
        })
        .collect::<Vec<_>>()
        .join("\n");

    if !html_invocation.is_empty() {
        Some(html_invocation)
    } else {
        None
    }
}
