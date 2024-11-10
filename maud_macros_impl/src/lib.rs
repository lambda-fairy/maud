#![doc(html_root_url = "https://docs.rs/maud_macros_impl/0.25.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

mod ast;
mod escape;
mod generate;
mod parse;
#[cfg(feature = "hotreload")]
mod runtime;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;

use crate::ast::Markup;

#[cfg(feature = "hotreload")]
use {
    crate::parse::parse_at_runtime, crate::runtime::format_str, proc_macro2::Literal,
    std::collections::HashMap,
};

pub use crate::escape::escape_to_string;

pub fn expand(input: TokenStream) -> TokenStream {
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let markups = parse::parse(input);

    expand_from_parsed(markups, size_hint)
}

fn expand_from_parsed(markups: Vec<Markup>, size_hint: usize) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", Span::mixed_site()));
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
    let markups = parse::parse(input.clone());
    expand_runtime_from_parsed(markups, "html!")
}

#[cfg(feature = "hotreload")]
fn expand_runtime_from_parsed(markups: Vec<Markup>, skip_to_keyword: &str) -> TokenStream {
    let stmts = runtime::generate(markups);

    let skip_to_keyword = TokenTree::Literal(Literal::string(skip_to_keyword));

    let tok = quote!({
        extern crate alloc;
        extern crate maud;
        let file_info = file!();
        let line_info = line!();

        let mut vars: ::std::collections::HashMap<&'static str, String> = ::std::collections::HashMap::new();
        let input = ::maud::macro_private::gather_html_macro_invocations(file_info, line_info, #skip_to_keyword);
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
    });

    let s = tok.to_string();

    if s.contains("unwrap_or_default") {
        // panic!("{}", s);
    }

    tok
}

#[cfg(feature = "hotreload")]
pub fn expand_runtime_main(
    vars: HashMap<&'static str, String>,
    input: Option<&str>,
    file_info: &str,
    line_info: u32,
) -> Result<String, String> {
    if let Some(input) = input {
        let res = ::std::panic::catch_unwind(|| parse_at_runtime(input.parse().unwrap()));

        if let Err(e) = res {
            if let Some(s) = e
                // Try to convert it to a String, then turn that into a str
                .downcast_ref::<String>()
                .map(String::as_str)
                // If that fails, try to turn it into a &'static str
                .or_else(|| {
                    e.downcast_ref::<&'static str>()
                        .map(::std::ops::Deref::deref)
                })
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
                Ok(template) => match template.render(&vars) {
                    Ok(template) => Ok(template),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
    } else {
        Err(format!(
            "can't find template source at {}:{}, please recompile",
            file_info, line_info
        ))
    }
}

/// Grabs the inside of an html! {} invocation and returns it as a string
pub fn gather_html_macro_invocations(
    file_path: &str,
    start_line: u32,
    skip_to_keyword: &str,
) -> Option<String> {
    let buf_reader = BufReader::new(File::open(file_path).unwrap());

    let mut braces_diff = 0;

    let html_invocation = buf_reader
        .lines()
        .skip(start_line as usize - 1)
        .map(|line| line.unwrap())
        // scan for beginning of the macro. start_line may point to it directly, but we want to
        // handle code flowing slightly downward.
        .skip_while(|line| !line.contains(skip_to_keyword))
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .take_while(|&c| {
            if c == '{' {
                braces_diff += 1;
            } else if c == '}' {
                braces_diff -= 1;
            }
            braces_diff != -1
        })
        .collect::<String>();

    if !html_invocation.is_empty() {
        Some(html_invocation)
    } else {
        None
    }
}
