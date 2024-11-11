#![doc(html_root_url = "https://docs.rs/maud_macros/0.26.0")]
// TokenStream values are reference counted, and the mental overhead of tracking
// lifetimes outweighs the marginal gains from explicit borrowing
#![allow(clippy::needless_pass_by_value)]

extern crate proc_macro;

use proc_macro_error2::proc_macro_error;

#[proc_macro]
#[proc_macro_error]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    maud_macros_impl::expand(input.into()).into()
}

#[cfg(feature = "hotreload")]
#[proc_macro]
#[proc_macro_error]
pub fn html_hotreload(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let x = maud_macros_impl::expand_runtime(input.into()).into();
    // panic!("{}", x);
    x
}
