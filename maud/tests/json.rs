#![cfg(feature = "json")]

use maud::{Render, html, json::Json};
use serde_json::{Value, json};

const EXAMPLE_HTML: &str = "</script><!--";
const EXAMPLE_HTML_JSON: &str = "\"\\u003c/script\\u003e\\u003c!--\"";

#[test]
fn render_value_escape_significant_chars() {
    let value = json!("<Fish & Chips>");
    let expected = "\"\\u003cFish \\u0026 Chips\\u003e\"";
    let observed = value.render().into_string();
    assert_eq!(expected, observed);
}

#[test]
fn render_value_parse_escaped_chars() {
    let value = json!("<Fish & Chips>");
    let rendered = value.render().into_string();
    let parsed: Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(value, parsed);
}

#[test]
fn render_value_cannot_break_out_of_script_tag() {
    let value = json!({ "html": EXAMPLE_HTML });
    let observed = html! { script { (value) } }.into_string();
    assert_eq!(
        format!("<script>{{\"html\":{EXAMPLE_HTML_JSON}}}</script>"),
        observed,
    );
}

#[derive(serde::Serialize)]
struct Name<'a>(&'a str);

#[test]
fn render_wrapper_escape_significant_chars() {
    let name = Name("<Fish & Chips>");
    let expected = "\"\\u003cFish \\u0026 Chips\\u003e\"";
    let observed = Json(name).render().into_string();
    assert_eq!(expected, observed);
}

#[test]
fn render_wrapper_parse_escaped_chars() {
    let name = Name("<Fish & Chips>");
    let rendered = Json(&name).render().into_string();
    let parsed: Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(name.0, parsed);
}

#[derive(serde::Serialize)]
struct Html<'a> {
    html: &'a str,
}

#[test]
fn render_wrapper_cannot_break_out_of_script_tag() {
    let value = Json(Html { html: EXAMPLE_HTML });
    let observed = html! { script { (value) } }.into_string();
    assert_eq!(
        format!("<script>{{\"html\":{EXAMPLE_HTML_JSON}}}</script>"),
        observed,
    );
}
