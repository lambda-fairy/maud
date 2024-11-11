#![doc(html_root_url = "https://docs.rs/maud_macros_impl/0.25.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate alloc;
use alloc::string::String;

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
    let markups = parse::parse(input.clone());

    expand_from_parsed(markups, size_hint)
}

fn expand_from_parsed(markups: Vec<Markup>, size_hint: usize) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__maud_output", Span::mixed_site()));
    let stmts = generate::generate(markups, output_ident.clone());
    quote!({
        extern crate maud;
        let mut #output_ident = ::maud::macro_private::String::with_capacity(#size_hint);
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
    expand_runtime_from_parsed(input, markups, "html!")
}

#[cfg(feature = "hotreload")]
fn expand_runtime_from_parsed(
    input: TokenStream,
    markups: Vec<Markup>,
    skip_to_keyword: &str,
) -> TokenStream {
    let vars_ident = TokenTree::Ident(Ident::new("__maud_vars", Span::mixed_site()));
    let skip_to_keyword = TokenTree::Literal(Literal::string(skip_to_keyword));
    let input_string = input.to_string();
    let original_input = TokenTree::Literal(Literal::string(&input_string));

    let stmts = runtime::generate(Some(vars_ident.clone()), markups);

    quote!({
        extern crate maud;

        let __maud_file_info = ::std::file!();
        let __maud_line_info = ::std::line!();

        let mut #vars_ident: ::maud::macro_private::HashMap<&'static str, ::maud::macro_private::String> = ::std::collections::HashMap::new();
        let __maud_input = ::maud::macro_private::gather_html_macro_invocations(
            __maud_file_info,
            __maud_line_info,
            #skip_to_keyword
        );

        let __maud_input = if let Some(ref input) = __maud_input {
            input
        } else {
            // fall back to original, unedited input when finding file info fails
            // TODO: maybe expose envvar to abort and force recompile?
            #original_input
        };

        #stmts;

        match ::maud::macro_private::expand_runtime_main(
            #vars_ident,
            __maud_input,
        ) {
            Ok(x) => ::maud::PreEscaped(x),
            Err(e) => ::maud::macro_private::render_runtime_error(&__maud_input, &e),
        }
    })
}

#[cfg(feature = "hotreload")]
pub fn expand_runtime_main(
    vars: HashMap<&'static str, String>,
    input: &str,
) -> Result<String, String> {
    let input: TokenStream = input.parse().unwrap_or_else(|_| panic!("{}", input));
    let res = ::std::panic::catch_unwind(|| parse_at_runtime(input.clone()));

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
        let markups = res.unwrap();
        let format_str = format_str(None, markups);

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
}

/// Grabs the inside of an html! {} invocation and returns it as a string
pub fn gather_html_macro_invocations(
    file_path: &str,
    start_line: u32,
    skip_to_keyword: &str,
) -> Option<String> {
    let buf_reader = BufReader::new(File::open(file_path).ok()?);

    let mut output = String::new();

    let mut lines_iter = buf_reader
        .lines()
        .skip(start_line as usize - 1)
        .map(|line| line.unwrap());

    const OPEN_BRACE: &[char] = &['[', '{', '('];
    const CLOSE_BRACE: &[char] = &[']', '}', ')'];

    // scan for beginning of the macro. start_line may point to it directly, but we want to
    // handle code flowing slightly downward.
    for line in &mut lines_iter {
        if let Some((_, after)) = line.split_once(skip_to_keyword) {
            // in case that the line is something inline like html! { .. }, we want to append the
            // rest of the line
            // skip ahead until first opening brace after match
            let after = if let Some((_, after2)) = after.split_once(OPEN_BRACE) {
                after2
            } else {
                after
            };

            output.push_str(after);
            break;
        }
    }

    let mut braces_diff = 0;

    'characterwise: for line in &mut lines_iter {
        for c in line.chars() {
            if OPEN_BRACE.contains(&c) {
                braces_diff += 1;
            } else if CLOSE_BRACE.contains(&c) {
                braces_diff -= 1;
            }

            if braces_diff == -1 {
                break 'characterwise;
            }

            output.push(c);
        }

        output.push('\n');
    }

    if !output.is_empty() {
        Some(output)
    } else {
        None
    }
}
