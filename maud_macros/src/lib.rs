#![doc(html_root_url = "https://docs.rs/maud_macros/0.23.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod escape;
mod generate;
mod parse;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use std::{
    env,
    ffi::OsStr,
    fmt::Display,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};
use syn::{parse_macro_input, LitStr};

#[proc_macro]
#[proc_macro_error]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

#[proc_macro]
#[proc_macro_error]
pub fn html_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expr = expand(input.into());
    println!("expansion:\n{}", expr);
    expr.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn html_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let orig_path = parse_macro_input!(input as LitStr);
    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let path = Path::new(&root).join(&orig_path.value());

    let file_name = path.file_name().unwrap_or_else(|| OsStr::new("unknown"));

    let mut file = abort_on_error(file_name, "error while opening", || {
        File::open(<PathBuf as AsRef<Path>>::as_ref(&path))
    });
    let mut file_contents = String::new();
    abort_on_error(file_name, "error while reading", || {
        file.read_to_string(&mut file_contents)
    });

    expand(abort_on_error(file_name, "error while parsing", || {
        TokenStream::from_str(&file_contents)
    }))
    .into()
}

fn abort_on_error<T, F, E>(file_name: &OsStr, description: &str, f: F) -> T
where
    F: FnOnce() -> Result<T, E>,
    E: Display,
{
    match f() {
        Ok(result) => result,
        Err(error) => abort_call_site!("{} {:?}: {}", description, file_name, error),
    }
}

fn expand(input: TokenStream) -> TokenStream {
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
