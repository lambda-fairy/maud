#![cfg(feature = "json")]

use maud::{json::Json, Render};
use serde_json::{json, Value};

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
