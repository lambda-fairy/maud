#![feature(plugin)]
#![feature(proc_macro)]

#![plugin(maud_lints)]

extern crate maud;

use maud::{html, html_to};

#[macro_use]
mod html_test;

#[test]
fn literals() {
    html_test!(assert_eq: "du\tcks-233.14\ngeese",
               markup: "du\tcks" "-23" "3.14\n" "geese");
}

#[test]
fn escaping() {
    html_test!(assert_eq: "&lt;flim&amp;flam&gt;",
               markup: "<flim&flam>");
}

#[test]
fn semicolons() {
    html_test!(assert_eq: "onetwothreefour",
               markup:
               "one";
               "two";
               "three";
               ;;;;;;;;;;;;;;;;;;;;;;;;
               "four";);
}

#[test]
fn blocks() {
    html_test!(assert_eq: "hello ducks geese swans",
               markup:
               "hello"
               {
                   " ducks" " geese"
               }
               " swans"
    );
}

#[test]
fn simple_elements() {
    html_test!(assert_eq: "<p><b>pickle</b>barrel<i>kumquat</i></p>",
               markup: p { b { "pickle" } "barrel" i { "kumquat" } });
}

#[test]
fn nesting_elements() {
    html_test!(assert_eq: "<html><body><div><p><sup>butts</sup></p></div></body></html>",
               markup: html body div p sup "butts");
}

#[test]
fn empty_elements() {
    html_test!(assert_eq: "pinkie<br>pie",
               markup: "pinkie" br; "pie");
}

#[test]
fn empty_elements_slash() {
    html_test!(assert_eq: "pinkie<br>pie",
               markup: "pinkie" br / "pie");
}

#[test]
fn simple_attributes() {
    html_test!(assert_eq:
               concat!(r#"<link rel="stylesheet" href="styles.css">"#,
                       r#"<section id="midriff"><p class="hotpink">Hello!</p></section>"#),
               markup:
               link rel="stylesheet" href="styles.css";
               section id="midriff" {
                   p class="hotpink" "Hello!"
               });
}

#[test]
fn empty_attributes() {
    html_test!(assert_eq: r#"<div readonly><input type="checkbox" checked></div>"#,
               markup: div readonly? input type="checkbox" checked?;);
}

#[test]
fn toggle_empty_attributes() {
    html_test!(bootstrap: { let rocks = true; },
               assert_eq:
               concat!(r#"<input checked>"#,
                       r#"<input>"#,
                       r#"<input checked>"#,
                       r#"<input>"#),
               markup:
               input checked?[true];
               input checked?[false];
               input checked?[rocks];
               input checked?[!rocks];
    );
}

#[test]
fn toggle_empty_attributes_braces() {
    html_test!(bootstrap: { struct Maud { rocks: bool } },
               assert_eq: r#"<input checked>"#,
               markup: input checked?[Maud { rocks: true }.rocks] /);
}

#[test]
fn colons_in_names() {
    html_test!(assert_eq: concat!(r#"<pon-pon:controls-alpha>"#,
                                  r#"<a on:click="yay()">Yay!</a>"#,
                                  r#"</pon-pon:controls-alpha>"#),
               markup: pon-pon:controls-alpha a on:click="yay()" "Yay!");
}

#[test]
fn hyphens_in_element_names() {
    html_test!(assert_eq: "<custom-element></custom-element>",
               markup: custom-element {});
}

#[test]
fn hyphens_in_attribute_names() {
    html_test!(assert_eq: r#"<this sentence-is="false" of-course></this>"#,
               markup: this sentence-is="false" of-course? {});
}

#[test]
fn class_shorthand() {
    html_test!(assert_eq: r#"<p>Hi, <span class="name">Lyra</span>!</p>"#,
               markup: p { "Hi, " span.name { "Lyra" } "!" });
}

#[test]
fn class_shorthand_with_space() {
    html_test!(assert_eq: r#"<p>Hi, <span class="name">Lyra</span>!</p>"#,
               markup: p { "Hi, " span .name { "Lyra" } "!" });
}

#[test]
fn classes_shorthand() {
    html_test!(assert_eq: r#"<p>Hi, <span class="name here">Lyra</span>!</p>"#,
               markup: p { "Hi, " span.name.here { "Lyra" } "!" });
}

#[test]
fn hyphens_in_class_names() {
    html_test!(assert_eq: r#"<p class="rocks-these are--my--rocks">yes</p>"#,
               markup: p.rocks-these.are--my--rocks "yes");
}

#[test]
fn toggle_classes() {
    macro_rules! toggle_classes {
        ($is_cupcake:expr, $is_muffin:expr, $a_eq:expr) => {
            html_test!(bootstrap: { let is_cupcake = $is_cupcake; let is_muffin = $is_muffin; },
                       assert_eq: $a_eq,
                       markup: p.cupcake[is_cupcake].muffin[is_muffin] "Testing!");
        }
    }
    toggle_classes!(true, true, r#"<p class="cupcake muffin">Testing!</p>"#);
    toggle_classes!(false, true, r#"<p class=" muffin">Testing!</p>"#);
    toggle_classes!(true, false, r#"<p class="cupcake">Testing!</p>"#);
    toggle_classes!(false, false, r#"<p class="">Testing!</p>"#);
}

#[test]
fn toggle_classes_braces() {
    html_test!(bootstrap: { struct Maud { rocks: bool } },
               assert_eq: r#"<p class="rocks">Awesome!</p>"#,
               markup: p.rocks[Maud { rocks: true }.rocks] "Awesome!");
}

#[test]
fn mixed_classes() {
    macro_rules! mixed_classes{
        ($is_muffin:expr, $a_eq:expr) => {
            html_test!(bootstrap: { let is_muffin = $is_muffin; },
                       assert_eq: $a_eq,
                       markup:p.cupcake.muffin[is_muffin].lamington "Testing!");
        }
    }
    mixed_classes!(true, r#"<p class="cupcake lamington muffin">Testing!</p>"#);
    mixed_classes!(false, r#"<p class="cupcake lamington">Testing!</p>"#);
}

#[test]
fn ids_shorthand() {
    html_test!(assert_eq: r#"<p>Hi, <span id="thing">Lyra</span>!</p>"#,
               markup: p { "Hi, " span#thing { "Lyra" } "!" });
}

#[test]
fn classes_attrs_ids_mixed_up() {
    html_test!(assert_eq: r#"<p>Hi, <span class="name here" id="thing" lang="en">Lyra</span>!</p>"#,
               markup: p { "Hi, " span.name.here lang="en" #thing { "Lyra" } "!" });
}
