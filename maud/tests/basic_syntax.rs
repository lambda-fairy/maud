use maud::{html, Markup};

#[test]
fn literals() {
    let s = html!("du\tcks" "-23" "3.14\n" "geese").into_string();
    assert_eq!(s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let s = html!("<flim&flam>").into_string();
    assert_eq!(s, "&lt;flim&amp;flam&gt;");
}

#[test]
fn semicolons() {
    let s = html! {
        "one";
        "two";
        "three";
        ;;;;;;;;;;;;;;;;;;;;;;;;
        "four";
    }
    .into_string();
    assert_eq!(s, "onetwothreefour");
}

#[test]
fn blocks() {
    let s = html! {
        "hello"
        {
            " ducks" " geese"
        }
        " swans"
    }
    .into_string();
    assert_eq!(s, "hello ducks geese swans");
}

#[test]
fn simple_elements() {
    let s = html!(p { b { "pickle" } "barrel" i { "kumquat" } }).into_string();
    assert_eq!(s, "<p><b>pickle</b>barrel<i>kumquat</i></p>");
}

#[test]
fn empty_elements() {
    let s = html!("pinkie" br; "pie").into_string();
    assert_eq!(s, "pinkie<br>pie");
}

#[test]
fn empty_elements_slash() {
    let s = html!("pinkie" br / "pie").into_string();
    assert_eq!(s, "pinkie<br>pie");
}

#[test]
fn simple_attributes() {
    let s = html! {
        link rel="stylesheet" href="styles.css";
        section id="midriff" {
            p class="hotpink" { "Hello!" }
        }
    }
    .into_string();
    assert_eq!(
        s,
        concat!(
            r#"<link rel="stylesheet" href="styles.css">"#,
            r#"<section id="midriff"><p class="hotpink">Hello!</p></section>"#
        )
    );
}

#[test]
fn empty_attributes() {
    let s = html!(div readonly? { input type="checkbox" checked?; }).into_string();
    assert_eq!(s, r#"<div readonly><input type="checkbox" checked></div>"#);
}

#[test]
fn toggle_empty_attributes() {
    let rocks = true;
    let s = html!({
        input checked?[true];
        input checked?[false];
        input checked?[rocks];
        input checked?[!rocks];
    })
    .into_string();
    assert_eq!(
        s,
        concat!(
            r#"<input checked>"#,
            r#"<input>"#,
            r#"<input checked>"#,
            r#"<input>"#
        )
    );
}

#[test]
fn toggle_empty_attributes_braces() {
    struct Maud {
        rocks: bool,
    }
    let s = html!(input checked?[Maud { rocks: true }.rocks] /).into_string();
    assert_eq!(s, r#"<input checked>"#);
}

#[test]
fn colons_in_names() {
    let s = html!(pon-pon:controls-alpha { a on:click="yay()" { "Yay!" } }).into_string();
    assert_eq!(
        s,
        concat!(
            r#"<pon-pon:controls-alpha>"#,
            r#"<a on:click="yay()">Yay!</a>"#,
            r#"</pon-pon:controls-alpha>"#
        )
    );
}

#[rustfmt::skip::macros(html)]
#[test]
fn hyphens_in_element_names() {
    let s = html!(custom-element {}).into_string();
    assert_eq!(s, "<custom-element></custom-element>");
}

#[test]
fn hyphens_in_attribute_names() {
    let s = html!(this sentence-is="false" of-course? {}).into_string();
    assert_eq!(s, r#"<this sentence-is="false" of-course></this>"#);
}

#[test]
fn class_shorthand() {
    let s = html!(p { "Hi, " span.name { "Lyra" } "!" }).into_string();
    assert_eq!(s, r#"<p>Hi, <span class="name">Lyra</span>!</p>"#);
}

#[test]
fn class_shorthand_with_space() {
    let s = html!(p { "Hi, " span .name { "Lyra" } "!" }).into_string();
    assert_eq!(s, r#"<p>Hi, <span class="name">Lyra</span>!</p>"#);
}

#[test]
fn classes_shorthand() {
    let s = html!(p { "Hi, " span.name.here { "Lyra" } "!" }).into_string();
    assert_eq!(s, r#"<p>Hi, <span class="name here">Lyra</span>!</p>"#);
}

#[test]
fn hyphens_in_class_names() {
    let s = html!(p.rocks-these.are--my--rocks { "yes" }).into_string();
    assert_eq!(s, r#"<p class="rocks-these are--my--rocks">yes</p>"#);
}

#[test]
fn class_string() {
    let s = html!(h1."pinkie-123" { "Pinkie Pie" }).into_string();
    assert_eq!(s, r#"<h1 class="pinkie-123">Pinkie Pie</h1>"#);
}

#[test]
fn toggle_classes() {
    fn test(is_cupcake: bool, is_muffin: bool) -> Markup {
        html!(p.cupcake[is_cupcake].muffin[is_muffin] { "Testing!" })
    }
    assert_eq!(
        test(true, true).into_string(),
        r#"<p class="cupcake muffin">Testing!</p>"#
    );
    assert_eq!(
        test(false, true).into_string(),
        r#"<p class=" muffin">Testing!</p>"#
    );
    assert_eq!(
        test(true, false).into_string(),
        r#"<p class="cupcake">Testing!</p>"#
    );
    assert_eq!(
        test(false, false).into_string(),
        r#"<p class="">Testing!</p>"#
    );
}

#[test]
fn toggle_classes_braces() {
    struct Maud {
        rocks: bool,
    }
    let s = html!(p.rocks[Maud { rocks: true }.rocks] { "Awesome!" }).into_string();
    assert_eq!(s, r#"<p class="rocks">Awesome!</p>"#);
}

#[test]
fn toggle_classes_string() {
    let is_cupcake = true;
    let is_muffin = false;
    let s = html!(p."cupcake"[is_cupcake]."is_muffin"[is_muffin] { "Testing!" }).into_string();
    assert_eq!(s, r#"<p class="cupcake">Testing!</p>"#);
}

#[test]
fn mixed_classes() {
    fn test(is_muffin: bool) -> Markup {
        html!(p.cupcake.muffin[is_muffin].lamington { "Testing!" })
    }
    assert_eq!(
        test(true).into_string(),
        r#"<p class="cupcake lamington muffin">Testing!</p>"#
    );
    assert_eq!(
        test(false).into_string(),
        r#"<p class="cupcake lamington">Testing!</p>"#
    );
}

#[test]
fn id_shorthand() {
    let s = html!(p { "Hi, " span#thing { "Lyra" } "!" }).into_string();
    assert_eq!(s, r#"<p>Hi, <span id="thing">Lyra</span>!</p>"#);
}

#[test]
fn id_string() {
    let s = html!(h1#"pinkie-123" { "Pinkie Pie" }).into_string();
    assert_eq!(s, r#"<h1 id="pinkie-123">Pinkie Pie</h1>"#);
}

#[test]
fn classes_attrs_ids_mixed_up() {
    let s = html!(p { "Hi, " span.name.here lang="en" #thing { "Lyra" } "!" }).into_string();
    assert_eq!(
        s,
        r#"<p>Hi, <span class="name here" id="thing" lang="en">Lyra</span>!</p>"#
    );
}

#[test]
fn div_shorthand_class() {
    let s = html!(.awesome-class {}).into_string();
    assert_eq!(s, r#"<div class="awesome-class"></div>"#);
}

#[test]
fn div_shorthand_id() {
    let s = html!(#unique-id {}).into_string();
    assert_eq!(s, r#"<div id="unique-id"></div>"#);
}

#[test]
fn div_shorthand_class_with_attrs() {
    let s = html!(.awesome-class contenteditable? dir="rtl" #unique-id {}).into_string();
    assert_eq!(
        s,
        r#"<div class="awesome-class" id="unique-id" contenteditable dir="rtl"></div>"#
    );
}

#[test]
fn div_shorthand_id_with_attrs() {
    let s = html!(#unique-id contenteditable? dir="rtl" .awesome-class {}).into_string();
    assert_eq!(
        s,
        r#"<div class="awesome-class" id="unique-id" contenteditable dir="rtl"></div>"#
    );
}
