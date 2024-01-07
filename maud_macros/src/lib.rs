#![doc(html_root_url = "https://docs.rs/maud_macros/0.26.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod escape;
mod generate;

use ast::DiagnosticParse;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    Error,
};

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

fn expand(input: TokenStream) -> TokenStream {
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();

    let mut diagnostics = Vec::new();
    let markups = match Parser::parse2(
        |input: ParseStream| ast::Markups::diagnostic_parse(input, &mut diagnostics),
        input,
    ) {
        Ok(data) => data,
        Err(err) => return err.to_compile_error(),
    };

    let mut diagnostics = diagnostics.into_iter();

    let error = if let Some(diag) = diagnostics.next() {
        let mut error = Error::from(diag);
        for diagnostic in diagnostics {
            error.combine(diagnostic.into());
        }
        Some(error.to_compile_error())
    } else {
        None
    };

    let output_ident = Ident::new("__maud_output", Span::mixed_site());
    let stmts = generate::generate(markups, output_ident.clone());
    quote! {{
        extern crate alloc;
        extern crate maud;
        let mut #output_ident = alloc::string::String::with_capacity(#size_hint);
        #stmts
        #error
        maud::PreEscaped(#output_ident)
    }}
}
