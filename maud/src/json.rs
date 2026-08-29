//! Format JSON strings for use in HTML, especially `<script>` tags.
//!
//! This module provides a [`Json`] wrapper type which will render its inner
//! value as HTML-safe JSON, and implementations of [`Render`] for
//! [`serde_json::Value`] and [`serde_json::value::RawValue`] as a convenience.
//!
//! # Notes
//!
//! This does *not* follow [WHATWG advice][advice-whatwg] for embedding content
//! into `<script>` tags, *nor* does it follow the [JSON-LD spec's
//! advice][advice-json-ld].
//!
//! The WHATWG's advice _for `<script>` elements_ suggests replacing `<`
//! characters with `\x3C`. This works when embedding JSON into a `<script>`
//! element that is being used for JavaScript because `\xNN` escapes are allowed
//! in JavaScript, but it does not work when embedding, say, JSON-LD into a
//! `<script>` element. These `\xNN` escape sequences are not recognised by JSON
//! parsers, and their use renders the JSON payload invalid.
//!
//! The JSON-LD spec suggests replacing several characters – `<`, `>`, `&`, and
//! quotes – with HTML entities. It follows that every receiving processor must
//! then reverse this transformation, but this does not seem to happen in
//! practice.
//!
//! Instead, this module replaces `<`, `>`, and `&` characters in JSON strings
//! (including object keys) with `\u003c`, `\u003e`, and `\u0026` Unicode escape
//! sequences respectively. This is understood by JSON parsers but is inert as
//! far as HTML `<script>` tags are concerned, and neutralises misinterpretation
//! of embedded HTML, XML, entities, and other `<…>` tagged content by HTML
//! processors. Notably, this also prevents HTML comments (i.e. `<!-- … -->`)
//! from being opened or prematurely closed by JSON within the `<script>` tag.
//! The JSON content is still entirely valid and able to be parsed as-is, with
//! no pre- or post-processing required; it could be copied and pasted from the
//! source document into a `.json` file and it would work.
//!
//! [advice-whatwg]:
//!     https://html.spec.whatwg.org/multipage/scripting.html#restrictions-for-contents-of-script-elements
//! [advice-json-ld]:
//!     https://www.w3.org/TR/json-ld/#restrictions-for-contents-of-json-ld-script-elements
//!
//! # Examples
//!
//! Safely passing a value from the server into a script in the page:
//!
//! ```rust
//! # use maud::{html, json::Json};
//! let foodstuff = "Fish & Chips";
//! let rendered = html! {
//!   script { "console.log(" (Json(foodstuff)) ")" }
//! };
//! assert_eq!(
//!   "<script>console.log(\"Fish \\u0026 Chips\")</script>",
//!   rendered.into_string(),
//! );
//! ```
//!
//! Embedding JSON-LD content into a page:
//!
//! ```rust
//! # use maud::html;
//! let json_ld_value = serde_json::json!({
//!   "@context": "https://schema.org",
//!   "@type": "BlogPosting",
//!   "mainEntityOfPage": { "@type": "WebPage", "@id": "https://example.org/json-ld" },
//!   "headline": "Fish & Chips",
//!   "description": "Fish and chips <with mushy peas> is a classic British meal.",
//!   "datePublished": "2025-10-19T19:06:03Z",
//!   "keywords": ["fish", "chips"],
//! });
//! let json_ld_element = html! {
//!   script type="application/ld+json" { (json_ld_value) }
//! }.into_string();
//! assert!(
//!   json_ld_element.contains(
//!     r#""description":"Fish and chips \u003cwith mushy peas\u003e "#
//!   ),
//!   "JSON-LD element: {json_ld_element}",
//! )
//! ```

use serde::Serialize;

use crate::Render;

/// A wrapper that renders the inner value as HTML-safe JSON.
///
/// Note that [`Render`] is implemented for [`serde_json::Value`] and
/// [`serde_json::value::RawValue`]. If you already have one of those, you don't
/// need this wrapper, though it will do no harm.
#[derive(Debug, Clone)]
pub struct Json<T: Serialize>(pub T);

impl<T: Serialize> Render for Json<T> {
    /// Appends an HTML-safe JSON representation of `self` to the given buffer.
    ///
    /// # Panics
    ///
    /// Panics if the JSON value cannot be serialized. `serde_json` says:
    ///
    /// > Serialization can fail if `T`’s implementation of `Serialize` decides
    /// > to fail, or if `T` contains a map with non-string keys.
    ///
    fn render_to(&self, buffer: &mut alloc::string::String) {
        render_to_buffer(&self.0, buffer);
    }
}

/// Implement [`Render`] for [`serde_json::Value`] directly.
///
/// Implementing for [`T: Serialize`][`Serialize`] is not possible because it's
/// too broad; [`Render`] is already implemented for several types that also
/// have [`Serialize`] implementations. The [`Json`] wrapper type is available
/// as a workaround.
impl Render for serde_json::Value {
    /// Appends an HTML-safe JSON representation of `self` to the given buffer.
    ///
    /// Serialization cannot fail here – the keys of a [`Map`][`serde_json::Map`]
    /// are always strings, and a [`Number`][`serde_json::Number`] is always
    /// finite – so, unlike [`Json`], this will not panic, beyond the usual
    /// allocation failure that any growing buffer is subject to.
    fn render_to(&self, buffer: &mut alloc::string::String) {
        render_to_buffer(self, buffer);
    }
}

/// Implement [`Render`] for [`serde_json::value::RawValue`] directly.
///
/// Implementing for [`T: Serialize`][`Serialize`] is not possible because it's
/// too broad; [`Render`] is already implemented for several types that also
/// have [`Serialize`] implementations. The [`Json`] wrapper type is available
/// as a workaround.
impl Render for serde_json::value::RawValue {
    /// Appends an HTML-safe JSON representation of `self` to the given buffer.
    ///
    /// Serialization cannot fail here – a
    /// [`RawValue`][`serde_json::value::RawValue`] is already-serialized JSON
    /// text – so, unlike [`Json`], this will not panic, beyond the usual
    /// allocation failure that any growing buffer is subject to.
    fn render_to(&self, buffer: &mut alloc::string::String) {
        render_to_buffer(self, buffer);
    }
}

/// Appends an HTML-safe JSON representation of `object` to the given buffer.
fn render_to_buffer<T: Serialize>(object: T, buffer: &mut alloc::string::String) {
    // SAFETY: We trust `serde_json` to produce UTF-8; see also the commentary
    // in `HtmlSafeJsonFormatter` for why it is UTF-8 safe.
    let mut writer = unsafe { buffer.as_mut_vec() };
    let mut serializer =
        serde_json::ser::Serializer::with_formatter(&mut writer, HtmlSafeJsonFormatter);
    object
        .serialize(&mut serializer)
        .expect("could not serialize JSON");
}

struct HtmlSafeJsonFormatter;

impl serde_json::ser::Formatter for HtmlSafeJsonFormatter {
    /// Writes a string fragment to the specified writer.
    ///
    /// It replaces `<`, `>`, and `&` characters with `\u003c`, `\u003e`, and
    /// `\u0026` Unicode escape sequences respectively.
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        write_escaped_fragment(writer, fragment)
    }

    /// Writes a raw fragment to the specified writer.
    ///
    /// It replaces `<`, `>`, and `&` characters with `\u003c`, `\u003e`, and
    /// `\u0026` Unicode escape sequences respectively.
    ///
    /// Unlike a string fragment, this receives an entire JSON document, and the
    /// scan is not aware of string boundaries. That is sound because none of
    /// these three characters can appear outside a string in well-formed JSON,
    /// which every safe constructor of
    /// [`RawValue`][`serde_json::value::RawValue`] guarantees.
    fn write_raw_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        write_escaped_fragment(writer, fragment)
    }
}

/// Writes a JSON fragment to the specified writer.
///
/// It replaces `<`, `>`, and `&` characters with `\u003c`, `\u003e`, and
/// `\u0026` Unicode escape sequences respectively.
///
/// For performance, it does this by scanning byte-wise for these three
/// characters and writing the string fragment in chunks. This maintains
/// valid UTF-8 output because these characters are all single-byte ASCII
/// characters and are unambiguous in a UTF-8 stream.
fn write_escaped_fragment<W>(writer: &mut W, fragment: &str) -> std::io::Result<()>
where
    W: ?Sized + std::io::Write,
{
    use serde_json::ser::{CharEscape::AsciiControl, CompactFormatter, Formatter};

    let mut offset = 0;
    for (index, byte) in fragment.bytes().enumerate() {
        if matches!(byte, b'<' | b'>' | b'&') {
            if index > offset {
                writer.write_all(fragment.as_bytes()[offset..index].as_ref())?;
            }
            CompactFormatter.write_char_escape(writer, AsciiControl(byte))?;
            offset = index + 1;
        }
    }
    if fragment.len() > offset {
        writer.write_all(fragment.as_bytes()[offset..].as_ref())?;
    }

    Ok(())
}
