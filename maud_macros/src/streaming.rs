use crate::ast::DiagnosticParse;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro2_diagnostics::Diagnostic;
use quote::quote;
use syn::parse::{ParseStream, Parser};

pub fn expand(input: TokenStream) -> TokenStream {
    // TODO: How to replace the size hint? Maybe measure the distance between
    // two yields?

    let mut diagnostics = Vec::new();
    let markups = match Parser::parse2(
        |input: ParseStream| crate::ast::Markups::diagnostic_parse(input, &mut diagnostics),
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
    let stmts = crate::generate::generate_streaming(markups, output_ident.clone());
    quote! {{
        extern crate alloc;
        extern crate maud;

        use maud::streaming::async_stream::stream;

        maud::streaming::StreamingMarkup(stream!{
            let mut #output_ident = alloc::string::String::new();

            #stmts
            #(#diag_tokens)*
            yield maud::PreEscaped(#output_ident);
        })
    }}
}
