#![cfg(feature = "json")]

use maud::{Render, html, json::Json};
use serde_json::{Value, json};

const EXAMPLE_TEXT: &str = "<Fish & Chips>";
const EXAMPLE_TEXT_JSON: &str = "\"\\u003cFish \\u0026 Chips\\u003e\"";
const EXAMPLE_HTML: &str = "</script><!--";
const EXAMPLE_HTML_JSON: &str = "\"\\u003c/script\\u003e\\u003c!--\"";

#[test]
fn render_value_escape_significant_chars() {
    let value = json!(EXAMPLE_TEXT);
    let expected = EXAMPLE_TEXT_JSON;
    let observed = value.render().into_string();
    assert_eq!(expected, observed);
}

#[test]
fn render_value_parse_escaped_chars() {
    let value = json!(EXAMPLE_TEXT);
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
    let name = Name(EXAMPLE_TEXT);
    let expected = EXAMPLE_TEXT_JSON;
    let observed = Json(name).render().into_string();
    assert_eq!(expected, observed);
}

#[test]
fn render_wrapper_parse_escaped_chars() {
    let name = Name(EXAMPLE_TEXT);
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

/// Tests for [`RawValue`][`serde_json::value::RawValue`] support.
pub mod raw_value {
    use maud::{Render, html, json::Json};
    use serde_json::{
        Value, json,
        value::{RawValue, to_raw_value},
    };

    use super::{EXAMPLE_HTML, EXAMPLE_HTML_JSON, EXAMPLE_TEXT, EXAMPLE_TEXT_JSON};

    #[test]
    fn render_raw_value_escape_significant_chars() {
        let value = json!(EXAMPLE_TEXT);
        let observed = to_raw_value(&value).unwrap().render().into_string();
        assert_eq!(EXAMPLE_TEXT_JSON, observed);
    }

    #[test]
    fn render_raw_value_parse_escaped_chars() {
        let value = json!(EXAMPLE_TEXT);
        let rendered = to_raw_value(&value).unwrap().render().into_string();
        let parsed: Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(value, parsed);
    }

    #[test]
    fn render_raw_value_cannot_break_out_of_script_tag() {
        let value = to_raw_value(&json!({ "html": EXAMPLE_HTML })).unwrap();
        let observed = html! { script { (value) } }.into_string();
        assert_eq!(
            format!("<script>{{\"html\":{EXAMPLE_HTML_JSON}}}</script>"),
            observed,
        );
    }

    #[test]
    fn render_raw_value_escapes_only_significant_chars() {
        let value = json!({ EXAMPLE_TEXT: [EXAMPLE_TEXT, 1, true, null] });
        let expected = format!("{{{EXAMPLE_TEXT_JSON}:[{EXAMPLE_TEXT_JSON},1,true,null]}}");
        let observed = to_raw_value(&value).unwrap().render().into_string();
        assert_eq!(expected, observed);
    }

    #[test]
    fn render_raw_value_does_not_reserialize() {
        let json_with_internal_whitespace = format!("{{\n  \"key\": \"{EXAMPLE_TEXT}\"\n}}");
        let value = RawValue::from_string(json_with_internal_whitespace).unwrap();
        let observed = value.render().into_string();
        assert_eq!(format!("{{\n  \"key\": {EXAMPLE_TEXT_JSON}\n}}"), observed);
    }

    #[derive(serde::Serialize)]
    struct Wrapper<'a> {
        inner: &'a RawValue,
    }

    #[test]
    fn render_wrapper_escape_nested_raw_value() {
        let inner = to_raw_value(&json!(EXAMPLE_TEXT)).unwrap();
        let observed = Json(Wrapper { inner: &inner }).render().into_string();
        assert_eq!(format!("{{\"inner\":{EXAMPLE_TEXT_JSON}}}"), observed);
    }
}
