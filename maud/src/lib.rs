//! Super fast HTML template engine.

use std::io::IoResult;

/// Utilities for escaping HTML5 markup.
///
/// These follow the *HTML fragment serialization algorithm*, as
/// specified by the [HTML 5.1 Working Draft][1].
///
/// [1]: http://www.w3.org/TR/html51/syntax.html#escapingString
pub mod escape {
    use super::render;
    use super::rt;

    /// Escape a double-quoted attribute value, as per HTML5 rules.
    pub fn attribute(s: &str) -> String {
        render(|w| {
            for c in s.chars() {
                try!(rt::escape_attribute(c, w));
            }
            Ok(())
        })
    }

    /// Escape non-attribute text content, as per HTML5 rules.
    pub fn non_attribute(s: &str) -> String {
        render(|w| {
            for c in s.chars() {
                try!(rt::escape_non_attribute(c, w));
            }
            Ok(())
        })
    }
}

/// Internal functions used by the `maud_macros` package. You should
/// never need to call these directly.
#[experimental = "These functions should not be called directly.
Use the macros in `maud_macros` instead."]
pub mod rt {
    use std::io::IoResult;

    #[inline]
    pub fn escape_attribute(c: char, w: &mut Writer) -> IoResult<()> {
        match c {
            '&' => w.write_str("&amp;"),
            '\u{A0}' => w.write_str("&nbsp;"),
            '"' => w.write_str("&quot;"),
            _ => w.write_char(c),
        }
    }

    #[inline]
    pub fn escape_non_attribute(c: char, w: &mut Writer) -> IoResult<()> {
        match c {
            '&' => w.write_str("&amp;"),
            '\u{A0}' => w.write_str("&nbsp;"),
            '<' => w.write_str("&lt;"),
            '>' => w.write_str("&gt;"),
            _ => w.write_char(c),
        }
    }
}

/// Render a template into a `String`.
pub fn render<F: FnOnce(&mut Writer) -> IoResult<()>>(template: F) -> String {
    let mut buf = vec![];
    template(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}
