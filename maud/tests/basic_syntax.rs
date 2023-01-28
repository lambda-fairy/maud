use maud::{html, Markup};

#[test]
fn literals() {
    let result = html! { "du\tcks" "-23" "3.14\n" "geese" };
    assert_eq!(result.into_string(), "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let result = html! { "<flim&flam>" };
    assert_eq!(result.into_string(), "&lt;flim&amp;flam&gt;");
}

#[test]
fn semicolons() {
    let result = html! {
        "one";
        "two";
        "three";
        ;;;;;;;;;;;;;;;;;;;;;;;;
        "four";
    };
    assert_eq!(result.into_string(), "onetwothreefour");
}

#[test]
fn blocks() {
    let result = html! {
        "hello"
        {
            " ducks" " geese"
        }
        " swans"
    };
    assert_eq!(result.into_string(), "hello ducks geese swans");
}

#[test]
fn simple_elements() {
    let result = html! { p { b { "pickle" } "barrel" i { "kumquat" } } };
    assert_eq!(
        result.into_string(),
        "<p><b>pickle</b>barrel<i>kumquat</i></p>"
    );
}

#[test]
fn empty_elements() {
    let result = html! { "pinkie" br; "pie" };
    assert_eq!(result.into_string(), "pinkie<br>pie");
}

#[test]
fn simple_attributes() {
    let result = html! {
        link rel="stylesheet" href="styles.css";
        section id="midriff" {
            p class="hotpink" { "Hello!" }
        }
    };
    assert_eq!(
        result.into_string(),
        concat!(
            r#"<link rel="stylesheet" href="styles.css">"#,
            r#"<section id="midriff"><p class="hotpink">Hello!</p></section>"#
        )
    );
}

#[test]
fn empty_attributes() {
    let result = html! { div readonly { input type="checkbox" checked; } };
    assert_eq!(
        result.into_string(),
        r#"<div readonly><input type="checkbox" checked></div>"#
    );
}

#[test]
fn toggle_empty_attributes() {
    let rocks = true;
    let result = html! {
        input checked[true];
        input checked[false];
        input checked[rocks];
        input checked[!rocks];
    };
    assert_eq!(
        result.into_string(),
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
    let result = html! { input checked[Maud { rocks: true }.rocks]; };
    assert_eq!(result.into_string(), r#"<input checked>"#);
}

#[test]
fn empty_attributes_question_mark() {
    let result = html! { input checked? disabled?[true]; };
    assert_eq!(result.into_string(), "<input checked disabled>");
}

#[test]
fn optional_attribute_some() {
    let result = html! { input value=[Some("value")]; };
    assert_eq!(result.into_string(), r#"<input value="value">"#);
}

#[test]
fn optional_attribute_none() {
    let result = html! { input value=[None as Option<&str>]; };
    assert_eq!(result.into_string(), r#"<input>"#);
}

#[test]
fn optional_attribute_non_string_some() {
    let result = html! { input value=[Some(42)]; };
    assert_eq!(result.into_string(), r#"<input value="42">"#);
}

#[test]
fn optional_attribute_variable() {
    let x = Some(42);
    let result = html! { input value=[x]; };
    assert_eq!(result.into_string(), r#"<input value="42">"#);
}

#[test]
fn optional_attribute_inner_value_evaluated_only_once() {
    let mut count = 0;
    html! { input value=[{ count += 1; Some("picklebarrelkumquat") }]; };
    assert_eq!(count, 1);
}

#[test]
fn optional_attribute_braces() {
    struct Pony {
        cuteness: Option<i32>,
    }
    let result = html! { input value=[Pony { cuteness: Some(9000) }.cuteness]; };
    assert_eq!(result.into_string(), r#"<input value="9000">"#);
}

#[test]
fn colons_in_names() {
    let result = html! { pon-pon:controls-alpha { a on:click="yay()" { "Yay!" } } };
    assert_eq!(
        result.into_string(),
        concat!(
            r#"<pon-pon:controls-alpha>"#,
            r#"<a on:click="yay()">Yay!</a>"#,
            r#"</pon-pon:controls-alpha>"#
        )
    );
}

#[test]
fn hyphens_in_element_names() {
    let result = html! { custom-element {} };
    assert_eq!(result.into_string(), "<custom-element></custom-element>");
}

#[test]
fn hyphens_in_attribute_names() {
    let result = html! { this sentence-is="false" of-course {} };
    assert_eq!(
        result.into_string(),
        r#"<this sentence-is="false" of-course></this>"#
    );
}

#[test]
fn class_shorthand() {
    let result = html! { p { "Hi, " span.name { "Lyra" } "!" } };
    assert_eq!(
        result.into_string(),
        r#"<p>Hi, <span class="name">Lyra</span>!</p>"#
    );
}

#[test]
fn class_shorthand_with_space() {
    let result = html! { p { "Hi, " span .name { "Lyra" } "!" } };
    assert_eq!(
        result.into_string(),
        r#"<p>Hi, <span class="name">Lyra</span>!</p>"#
    );
}

#[test]
fn classes_shorthand() {
    let result = html! { p { "Hi, " span.name.here { "Lyra" } "!" } };
    assert_eq!(
        result.into_string(),
        r#"<p>Hi, <span class="name here">Lyra</span>!</p>"#
    );
}

#[test]
fn hyphens_in_class_names() {
    let result = html! { p.rocks-these.are--my--rocks { "yes" } };
    assert_eq!(
        result.into_string(),
        r#"<p class="rocks-these are--my--rocks">yes</p>"#
    );
}

#[test]
fn class_string() {
    let result = html! { h1."pinkie-123" { "Pinkie Pie" } };
    assert_eq!(
        result.into_string(),
        r#"<h1 class="pinkie-123">Pinkie Pie</h1>"#
    );
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
    let result = html! { p.rocks[Maud { rocks: true }.rocks] { "Awesome!" } };
    assert_eq!(result.into_string(), r#"<p class="rocks">Awesome!</p>"#);
}

#[test]
fn toggle_classes_string() {
    let is_cupcake = true;
    let is_muffin = false;
    let result = html! { p."cupcake"[is_cupcake]."is_muffin"[is_muffin] { "Testing!" } };
    assert_eq!(result.into_string(), r#"<p class="cupcake">Testing!</p>"#);
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
    let result = html! { p { "Hi, " span #thing { "Lyra" } "!" } };
    assert_eq!(
        result.into_string(),
        r#"<p>Hi, <span id="thing">Lyra</span>!</p>"#
    );
}

#[test]
fn id_string() {
    let result = html! { h1 #"pinkie-123" { "Pinkie Pie" } };
    assert_eq!(
        result.into_string(),
        r#"<h1 id="pinkie-123">Pinkie Pie</h1>"#
    );
}

#[test]
fn classes_attrs_ids_mixed_up() {
    let result = html! { p { "Hi, " span.name.here lang="en" #thing { "Lyra" } "!" } };
    assert_eq!(
        result.into_string(),
        r#"<p>Hi, <span class="name here" id="thing" lang="en">Lyra</span>!</p>"#
    );
}

#[test]
fn div_shorthand_class() {
    let result = html! { .awesome-class {} };
    assert_eq!(result.into_string(), r#"<div class="awesome-class"></div>"#);
}

#[test]
fn div_shorthand_id() {
    let result = html! { #unique-id {} };
    assert_eq!(result.into_string(), r#"<div id="unique-id"></div>"#);
}

#[test]
fn div_shorthand_class_with_attrs() {
    let result = html! { .awesome-class contenteditable dir="rtl" #unique-id {} };
    assert_eq!(
        result.into_string(),
        r#"<div class="awesome-class" id="unique-id" contenteditable dir="rtl"></div>"#
    );
}

#[test]
fn div_shorthand_id_with_attrs() {
    let result = html! { #unique-id contenteditable dir="rtl" .awesome-class {} };
    assert_eq!(
        result.into_string(),
        r#"<div class="awesome-class" id="unique-id" contenteditable dir="rtl"></div>"#
    );
}
