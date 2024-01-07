#![doc(html_root_url = "https://docs.rs/maud_macros/0.26.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

mod ast;
mod escape;
mod generate;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let markups = parse_macro_input!(input as ast::Markups);

    expand(size_hint, markups).into()
}

fn expand(size_hint: usize, markups: ast::Markups) -> TokenStream {
    let output_ident = Ident::new("__maud_output", Span::mixed_site());
    let stmts = generate::generate(markups, output_ident.clone());
    quote! {{
        extern crate alloc;
        extern crate maud;
        let mut #output_ident = alloc::string::String::with_capacity(#size_hint);
        #stmts
        maud::PreEscaped(#output_ident)
    }}
}
