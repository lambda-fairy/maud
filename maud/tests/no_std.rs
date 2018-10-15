#![no_std]
#![feature(alloc, proc_macro_hygiene)]

extern crate alloc;
extern crate maud;

use maud::html;

#[test]
fn issue_13() {
    use alloc::string::String;
    let owned = String::from("yay");
    let _ = html!((owned));
    // Make sure the `html!` call didn't move it
    let _owned = owned;
}
