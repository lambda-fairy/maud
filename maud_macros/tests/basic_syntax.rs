#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;

#[test]
fn literals() {
    let mut s = String::new();
    html!(s, "du\tcks" "-23" "3.14\n" "geese").unwrap();
    assert_eq!(s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let mut s = String::new();
    html!(s, "<flim&flam>").unwrap();
    assert_eq!(s, "&lt;flim&amp;flam&gt;");
}

#[test]
fn semicolons() {
    let mut s = String::new();
    html!(s, {
        "one";
        "two";
        "three";
        ;;;;;;;;;;;;;;;;;;;;;;;;
        "four";
    }).unwrap();
    assert_eq!(s, "onetwothreefour");
}

#[test]
fn blocks() {
    let mut s = String::new();
    html!(s, {
        "hello"
        {
            " ducks" " geese"
        }
        " swans"
    }).unwrap();
    assert_eq!(s, "hello ducks geese swans");
}

#[test]
fn simple_elements() {
    let mut s = String::new();
    html!(s, p { b { "pickle" } "barrel" i { "kumquat" } }).unwrap();
    assert_eq!(s, "<p><b>pickle</b>barrel<i>kumquat</i></p>");
}

#[test]
fn nesting_elements() {
    let mut s = String::new();
    html!(s, html body div p sup "butts").unwrap();
    assert_eq!(s, "<html><body><div><p><sup>butts</sup></p></div></body></html>");
}

#[test]
fn empty_elements() {
    let mut s = String::new();
    html!(s, "pinkie" br/ "pie").unwrap();
    assert_eq!(s, "pinkie<br>pie");
}

#[test]
fn simple_attributes() {
    let mut s = String::new();
    html!(s, {
        link rel="stylesheet" href="styles.css"/
        section id="midriff" {
            p class="hotpink" "Hello!"
        }
    }).unwrap();
    assert_eq!(s, concat!(
            r#"<link rel="stylesheet" href="styles.css">"#,
            r#"<section id="midriff"><p class="hotpink">Hello!</p></section>"#));
}

#[test]
fn empty_attributes() {
    let mut s = String::new();
    html!(s, div readonly? input type="checkbox" checked? /).unwrap();
    assert_eq!(s, r#"<div readonly><input type="checkbox" checked></div>"#);
}

#[test]
fn colons_in_names() {
    let mut s = String::new();
    html!(s, pon-pon:controls-alpha a on:click="yay()" "Yay!").unwrap();
    assert_eq!(s, concat!(
            r#"<pon-pon:controls-alpha>"#,
            r#"<a on:click="yay()">Yay!</a>"#,
            r#"</pon-pon:controls-alpha>"#));
}

#[test]
fn hyphens_in_element_names() {
    let mut s = String::new();
    html!(s, custom-element {}).unwrap();
    assert_eq!(s, "<custom-element></custom-element>");
}

#[test]
fn hyphens_in_attribute_names() {
    let mut s = String::new();
    html!(s, this sentence-is="false" of-course? {}).unwrap();
    assert_eq!(s, r#"<this sentence-is="false" of-course></this>"#);
}

#[test]
fn class_shorthand() {
    let mut s = String::new();
    html!(s, p { "Hi, " span.name { "Lyra" } "!" }).unwrap();
    assert_eq!(s, r#"<p>Hi, <span class="name">Lyra</span>!</p>"#);
}

#[test]
fn class_shorthand_with_space() {
    let mut s = String::new();
    html!(s, p { "Hi, " span .name { "Lyra" } "!" }).unwrap();
    assert_eq!(s, r#"<p>Hi, <span class="name">Lyra</span>!</p>"#);
}

#[test]
fn classes_shorthand() {
    let mut s = String::new();
    html!(s, p { "Hi, " span.name.here { "Lyra" } "!" }).unwrap();
    assert_eq!(s, r#"<p>Hi, <span class="name here">Lyra</span>!</p>"#);
}

#[test]
fn hyphens_in_class_names() {
    let mut s = String::new();
    html!(s, p.rocks-these.are--my--rocks "yes").unwrap();
    assert_eq!(s, r#"<p class="rocks-these are--my--rocks">yes</p>"#);
}

#[test]
fn ids_shorthand() {
    let mut s = String::new();
    html!(s, p { "Hi, " span#thing { "Lyra" } "!" }).unwrap();
    assert_eq!(s, r#"<p>Hi, <span id="thing">Lyra</span>!</p>"#);
}

#[test]
fn classes_attrs_ids_mixed_up() {
    let mut s = String::new();
    html!(s, p { "Hi, " span.name.here lang="en" #thing { "Lyra" } "!" }).unwrap();
    assert_eq!(s, "<p>Hi, <span lang=\"en\" class=\"name here\" id=\"thing\">Lyra</span>!</p>");
}
