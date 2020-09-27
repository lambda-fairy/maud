// Make sure `std` is available but the prelude isn't
#![no_std]
extern crate std;

use maud::html;

#[test]
fn issue_170() {
    let number = 42;
    let _ = html! { (number) };
}
