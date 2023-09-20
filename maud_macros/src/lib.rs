#![doc(html_root_url = "https://docs.rs/maud_macros/0.25.0")]
extern crate proc_macro;

use proc_macro_error::proc_macro_error;

#[proc_macro]
#[proc_macro_error]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    maud_macros_impl::expand(input.into()).into()
}

#[proc_macro]
#[proc_macro_error]
pub fn html_hotreload(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    maud_macros_impl::expand_runtime(input.into()).into()
}
