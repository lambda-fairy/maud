#![feature(plugin)]
#![feature(proc_macro)]

#![plugin(maud_lints)]

extern crate maud;

use maud::{html, html_to};

#[macro_use]
mod html_test;

#[test]
fn literals() {
    html_test!(assert_eq: "&lt;pinkie&gt;",
               markup: ("<pinkie>"));
}

#[test]
fn raw_literals() {
    html_test!(bootstrap: { use maud::PreEscaped; },
               assert_eq: "<pinkie>",
               markup: (PreEscaped("<pinkie>")));
}

#[test]
fn blocks() {
    html_test!(assert_eq: "3628800",
               markup:
               ({
                   let mut result = 1i32;
                   for i in 2..11 {
                       result *= i;
                   }
                   result
               })
    );
}

#[test]
fn attributes() {
    html_test!(bootstrap: { let alt = "Pinkie Pie"; },
               assert_eq: r#"<img src="pinkie.jpg" alt="Pinkie Pie">"#,
               markup: img src="pinkie.jpg" alt=(alt) /);
}

static BEST_PONY: &'static str = "Pinkie Pie";

#[test]
fn statics() {
    html_test!(assert_eq: "Pinkie Pie",
               markup: (BEST_PONY));
}

#[test]
fn locals() {
    html_test!(bootstrap: { let best_pony = "Pinkie Pie"; },
               assert_eq: "Pinkie Pie",
               markup: (best_pony));
}

/// An example struct, for testing purposes only
struct Creature {
    name: &'static str,
    /// Rating out of 10, where:
    /// * 0 is a naked mole rat with dysentery
    /// * 10 is Sweetie Belle in a milkshake
    adorableness: u32,
}

impl Creature {
    fn repugnance(&self) -> u32 {
        10 - self.adorableness
    }
}

#[test]
fn structs() {
    html_test!(bootstrap: {
                   let pinkie = Creature {
                       name: "Pinkie Pie",
                       adorableness: 9,
                   };
               },
               assert_eq: "Name: Pinkie Pie. Rating: 1",
               markup:
               "Name: " (pinkie.name) ". Rating: " (pinkie.repugnance()));
}

#[test]
fn tuple_accessors() {
    html_test!(bootstrap: { let a = ("ducks", "geese"); },
               assert_eq: "ducks",
               markup: (a.0));
}

#[test]
fn splice_with_path() {
    html_test!(bootstrap: {
                   mod inner {
                       pub fn name() -> &'static str {
                           "Maud"
                       }
                   }
               },
               assert_eq: "Maud",
               markup: (inner::name()));
}

#[test]
fn nested_macro_invocation() {
    html_test!(bootstrap: { let best_pony = "Pinkie Pie"; },
               assert_eq: "Pinkie Pie is best pony",
               markup: (format!("{} is best pony", best_pony)));
}
