#![feature(proc_macro_hygiene)]

extern crate maud_macros;

use maud_macros::html;

fn main() {
    let markup = html!{ 
        if {} //~WARNING found keyword `if` - should this be a `@if`?
    };
}
