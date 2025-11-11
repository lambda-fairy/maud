#![doc(html_root_url = "https://docs.rs/maud_macros/0.27.0")]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod escape;
mod generate;

use ast::DiagnosticParse;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro2_diagnostics::Diagnostic;
use quote::quote;
use syn::parse::{ParseStream, Parser};

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into(), false).into()
}

#[proc_macro]
pub fn html_render(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into(), true).into()
}

fn expand(input: TokenStream, as_struct: bool) -> TokenStream {
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();

    let mut diagnostics = Vec::new();
    let markups = match Parser::parse2(
        |input: ParseStream| ast::Markups::diagnostic_parse(input, &mut diagnostics),
        input,
    ) {
        Ok(data) => data,
        Err(err) => {
            let err = err.to_compile_error();
            let diag_tokens = diagnostics.into_iter().map(Diagnostic::emit_as_expr_tokens);

            return quote! {{
                #err
                #(#diag_tokens)*
            }};
        }
    };

    let diag_tokens = diagnostics.into_iter().map(Diagnostic::emit_as_expr_tokens);

    let output_ident = Ident::new("__maud_output", Span::mixed_site());
    let stmts = generate::generate(markups, output_ident.clone(), as_struct);
    if as_struct {
        quote! {{
            extern crate alloc;
            extern crate maud;

            maud::macro_private::RenderFn({
                #[inline(always)]
                |#output_ident: &mut String| {
                    #stmts
                    #(#diag_tokens)*
                }
            })
        }}
    } else {
        quote! {{
            extern crate alloc;
            extern crate maud;
            let mut #output_ident = alloc::string::String::with_capacity(#size_hint);
            #stmts
            #(#diag_tokens)*
            maud::PreEscaped(#output_ident)
        }}
    }
}
