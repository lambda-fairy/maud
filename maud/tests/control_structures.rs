#![feature(conservative_impl_trait)]
#![feature(plugin)]
#![feature(proc_macro)]

#![plugin(maud_lints)]

extern crate maud;

use maud::{html, html_to};

#[macro_use]
mod html_test;

#[test]
fn if_expr() {
    html_test!(bootstrap: {
                   for (number, &name) in
                       (1..4).zip(["one", "two", "three"].iter())
               },
               assert_eq: name,
               markup:
               @if number == 1 {
                   "one"
               } @else if number == 2 {
                   "two"
               } @else if number == 3 {
                   "three"
               } @else {
                   "oh noes"
               }
    );
}

#[test]
fn if_let() {
    html_test!(bootstrap: {
                   for &(input, output) in &[(Some("yay"), "yay"),
                                             (None, "oh noes")]
               },
               assert_eq: output,
               markup:
               @if let Some(value) = input {
                   (value)
               } @else {
                   "oh noes"
               }
    );
}

#[test]
fn while_expr() {
    html_test!(bootstrap: { let mut numbers = (0..3).into_iter().peekable(); },
               assert_eq: "<ul><li>0</li><li>1</li><li>2</li></ul>",
               markup:
               ul @while numbers.peek().is_some() {
                   li (numbers.next().unwrap())
               }
    );
}

#[test]
fn while_let_expr() {
    html_test!(bootstrap: {
                   let mut numbers = (0..3).into_iter();
                   #[cfg_attr(feature = "cargo-clippy", allow(while_let_on_iterator))]
               },
               assert_eq: "<ul><li>0</li><li>1</li><li>2</li></ul>",
               markup:
               ul @while let Some(n) = numbers.next() {
                   li (n)
               }
    );
}

#[test]
fn for_expr() {
    html_test!(bootstrap: { let ponies = ["Apple Bloom", "Scootaloo", "Sweetie Belle"]; },
               assert_eq: concat!("<ul>",
                                  "<li>Apple Bloom</li>",
                                  "<li>Scootaloo</li>",
                                  "<li>Sweetie Belle</li>",
                                  "</ul>"),
               markup:
               ul @for pony in &ponies {
                   li (pony)
               }
    );
}

#[test]
fn match_expr() {
    html_test!(bootstrap: {
                   for &(input, output) in &[(Some("yay"), "<div>yay</div>"),
                                             (None, "oh noes")]
               },
               assert_eq: output,
               markup:
               @match input {
                   Some(value) => {
                       div (value)
                   },
                   None => {
                       "oh noes"
                   },
               }
    );
}

#[test]
fn match_expr_without_delims() {
    html_test!(bootstrap: {
                   for &(input, output) in &[(Some("yay"), "yay"),
                                             (None, "<span>oh noes</span>")]
               },
               assert_eq: output,
               markup:
               @match input {
                   Some(value) => (value),
                   None => span "oh noes",
               }
    );
}

#[test]
fn match_expr_with_guards() {
    html_test!(bootstrap: {
                   for &(input, output) in &[(Some(1), "one"),
                                             (None, "none"),
                                             (Some(2), "2")]
               },
               assert_eq: output,
               markup:
               @match input {
                   Some(value) if value == 1 => "one",
                   Some(value) => (value),
                   None => "none",
               }
    );
}

#[test]
fn match_in_attribute() {
    html_test!(bootstrap: {
                   for &(input, output) in &[(1, "<span class=\"one\">1</span>"),
                                             (2, "<span class=\"two\">2</span>"),
                                             (3, "<span class=\"many\">3</span>")]
               },
               assert_eq: output,
               markup:
            span class=@match input {
                1 => "one",
                2 => "two",
                _ => "many",
            } { (input) }
    );
}

#[test]
fn let_expr() {
    html_test!(assert_eq: "I have 42 cupcakes!",
               markup:
               @let x = 42;
               "I have " (x) " cupcakes!"
    );
}

#[test]
fn let_lexical_scope() {
    html_test!(bootstrap: { let x = 42; },
               assert_eq: concat!("Twilight thought I had 99 cupcakes, ",
                                  "but I only had 42."),
               markup:
               {
                   @let x = 99;
                   "Twilight thought I had " (x) " cupcakes, "
               }
               "but I only had " (x) "."
    );
}

#[test]
fn let_type_ascription() {
    html_test!(assert_eq: "I have 42 cupcakes!",
               markup:
               @let mut x: Box<Iterator<Item=u32>> = Box::new(vec![42].into_iter());
               "I have " (x.next().unwrap()) " cupcakes!"
    );
}
