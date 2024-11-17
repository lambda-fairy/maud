//! Helper crate for `maud` and `maud_macros`. Nothing in this crate is semver-compliant, and is
//! not meant for direct consumption.
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
    path::Path,
};

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;

use crate::ast::Markup;

#[cfg(feature = "hotreload")]
use {crate::parse::parse_at_runtime, proc_macro2::Literal, std::collections::HashMap};

#[cfg(feature = "hotreload")]
pub use crate::runtime::PartialTemplate;

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
    let input_string = input.to_string();
    let markups = parse::parse(input.clone());
    let partial_template = expand_runtime_from_parsed(markups);
    let original_input = TokenTree::Literal(Literal::string(&input_string));

    quote! {{
        extern crate maud;

        let __maud_file_info = ::std::file!();
        let __maud_line_info = ::std::line!();

        let __maud_input = ::maud::macro_private::gather_html_macro_invocations(
            __maud_file_info,
            __maud_line_info,
        );

        let __maud_input = match __maud_input {
            Ok(x) => x,
            Err(e) => {
                if ::maud::macro_private::env_var("MAUD_SOURCE_NO_FALLBACK").as_deref() == Ok("1") {
                    panic!("failed to find sourcecode for {}:{}, error: {:?}", __maud_file_info, __maud_line_info, e);
                }

                // fall back to original, unedited input when finding file info fails
                #original_input.parse().unwrap()
            }
        };

        match #partial_template(::maud::macro_private::Vec::from([__maud_input.clone()])) {
            Ok(x) => ::maud::PreEscaped(x),
            Err(e) => ::maud::macro_private::render_runtime_error(&e),
        }
    }}
}

#[cfg(feature = "hotreload")]
fn expand_runtime_from_parsed(markups: Vec<Markup>) -> TokenStream {
    let vars_ident = TokenTree::Ident(Ident::new("__maud_vars", Span::mixed_site()));
    let stmts = runtime::generate(Some(vars_ident.clone()), markups);

    quote!({
        let mut #vars_ident: ::maud::macro_private::HashMap<&'static str, ::maud::macro_private::PartialTemplate> = ::std::default::Default::default();

        #stmts

        let f : ::maud::macro_private::PartialTemplate = ::maud::macro_private::Box::new(move |mut sources| {
            assert!(sources.len() == 1);
            let input = sources.pop().unwrap();
            ::maud::macro_private::expand_runtime_main(
                #vars_ident,
                input,
            )
        });

        f
    })
}

#[cfg(feature = "hotreload")]
pub fn expand_runtime_main(
    vars: HashMap<&'static str, PartialTemplate>,
    input: TokenStream,
) -> Result<String, String> {
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
            return Err(format!("{}, source: {}", s, input));
        } else {
            return Err("unknown panic".to_owned());
        }
    } else {
        let markups = res.unwrap();
        let interpreter = runtime::build_interpreter(markups);
        interpreter.run(vars)
    }
}

/// Grabs the inside of an html! {} invocation and returns it as a string
pub fn gather_html_macro_invocations(
    file_path: &str,
    start_line: u32,
) -> Result<TokenStream, String> {
    let mut errors = String::new();
    let mut file = None;

    for path in [
        // try a few paths to deal with workspaces. insta has a more sophisticated, complete
        // version of this
        Path::new(file_path).to_owned(),
        Path::new("../").join(file_path),
    ] {
        let path = std::path::absolute(path).unwrap();
        match File::open(&path) {
            Ok(f) => {
                file = Some(f);
                break;
            }
            Err(e) => {
                errors.push_str(&e.to_string());
                errors.push('\n');
            }
        }
    }

    let file = match file {
        Some(x) => x,
        None => return Err(errors),
    };

    let buf_reader = BufReader::new(file);

    let mut output = String::new();

    let mut lines_iter = buf_reader
        .lines()
        .skip(start_line as usize - 1)
        .map(|line| line.unwrap());

    let mut rest_of_line = String::new();
    let mut braces_diff = 0;

    fn track_braces(c: char) -> i32 {
        match c {
            '[' | '{' | '(' => 1,
            ']' | '}' | ')' => -1,
            _ => 0,
        }
    }

    // scan for beginning of the macro. start_line may point to it directly, but we want to
    // handle code flowing slightly downward.
    for line in &mut lines_iter {
        for c in line.chars() {
            braces_diff += track_braces(c);
            if braces_diff < 0 {
                return Err("too many closing braces".to_owned());
            }
        }

        if let Some((_, after)) = line.split_once("html!") {
            let after = if let Some((_, after2)) = after.split_once(&['[', '{', '(']) {
                after2
            } else {
                after
            };

            rest_of_line.push_str(after);
            break;
        }
    }

    braces_diff = 0;

    'linewise: for line in Some(rest_of_line).into_iter().chain(lines_iter) {
        for c in line.chars() {
            braces_diff += track_braces(c);
            if braces_diff == -1 {
                break 'linewise;
            }
            output.push(c);
        }
        output.push('\n');
    }

    let output = output.trim();

    if output.is_empty() {
        return Err("output is empty".to_string());
    }

    if output.starts_with("///") {
        // line/file information in doctests is 100% wrong and will lead to catastrophic results.
        return Err("cannot handle livereload in doctests".to_string());
    }

    output
        .parse()
        .map_err(|e| format!("failed to parse output: {}", e))
}
