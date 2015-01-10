//! Super fast HTML template engine.

#![allow(unstable)]

use std::fmt;
use std::fmt::Writer as FmtWriter;

pub type FmtResult<T> = Result<T, fmt::Error>;

/// Utilities for escaping HTML5 markup.
///
/// These follow the *HTML fragment serialization algorithm*, as
/// specified by the [HTML 5.1 Working Draft][1].
///
/// [1]: http://www.w3.org/TR/html51/syntax.html#escapingString
pub mod escape {
    use std::fmt::Writer as FmtWriter;

    use super::render;
    use super::rt;

    /// Escape a double-quoted attribute value, as per HTML5 rules.
    pub fn attribute(s: &str) -> String {
        render(|w| rt::escape_attribute(w, |w| w.write_str(s)))
    }

    /// Escape non-attribute text content, as per HTML5 rules.
    pub fn non_attribute(s: &str) -> String {
        render(|w| rt::escape_non_attribute(w, |w| w.write_str(s)))
    }
}

/// Internal functions used by the `maud_macros` package. You should
/// never need to call these directly.
#[experimental = "These functions should not be called directly.
Use the macros in `maud_macros` instead."]
pub mod rt {
    use std::fmt::Writer as FmtWriter;
    use super::FmtResult;

    struct AttrEscaper<'a, 'b: 'a> {
        inner: &'a mut (FmtWriter + 'b),
    }

    impl<'a, 'b> FmtWriter for AttrEscaper<'a, 'b> {
        fn write_str(&mut self, s: &str) -> FmtResult<()> {
            for c in s.chars() {
                try!(match c {
                    '&' => self.inner.write_str("&amp;"),
                    '\u{A0}' => self.inner.write_str("&nbsp;"),
                    '"' => self.inner.write_str("&quot;"),
                    _ => write!(self.inner, "{}", c),
                });
            }
            Ok(())
        }
    }

    struct NonAttrEscaper<'a, 'b: 'a> {
        inner: &'a mut (FmtWriter + 'b),
    }

    impl<'a, 'b> FmtWriter for NonAttrEscaper<'a, 'b> {
        fn write_str(&mut self, s: &str) -> FmtResult<()> {
            for c in s.chars() {
                try!(match c {
                    '&' => self.inner.write_str("&amp;"),
                    '\u{A0}' => self.inner.write_str("&nbsp;"),
                    '<' => self.inner.write_str("&lt;"),
                    '>' => self.inner.write_str("&gt;"),
                    _ => write!(self.inner, "{}", c),
                });
            }
            Ok(())
        }
    }

    #[inline]
    pub fn escape_attribute<F>(w: &mut FmtWriter, f: F) -> FmtResult<()> where
        F: FnOnce(&mut FmtWriter) -> FmtResult<()>
    {
        f(&mut AttrEscaper { inner: w })
    }

    #[inline]
    pub fn escape_non_attribute<F>(w: &mut FmtWriter, f: F) -> FmtResult<()> where
        F: FnOnce(&mut FmtWriter) -> FmtResult<()>
    {
        f(&mut NonAttrEscaper { inner: w })
    }
}

/// Render a template into a `String`.
pub fn render<F>(template: F) -> String where
    F: FnOnce(&mut FmtWriter) -> FmtResult<()>
{
    let mut buf = String::new();
    template(&mut buf).unwrap();
    buf
}
