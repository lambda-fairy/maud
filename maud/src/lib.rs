//! Super fast HTML template engine.

#![allow(unstable)]

use std::fmt;
use std::fmt::Writer as FmtWriter;

pub type FmtResult<T> = Result<T, fmt::Error>;

/// Escape an HTML value.
pub fn escape(s: &str) -> String {
    render(|w| rt::escape(w, |w| w.write_str(s)))
}

/// Internal functions used by the `maud_macros` package. You should
/// never need to call these directly.
#[experimental = "These functions should not be called directly.
Use the macros in `maud_macros` instead."]
pub mod rt {
    use std::fmt::Writer as FmtWriter;
    use super::FmtResult;

    struct Escaper<'a, 'b: 'a> {
        inner: &'a mut (FmtWriter + 'b),
    }

    impl<'a, 'b> FmtWriter for Escaper<'a, 'b> {
        fn write_str(&mut self, s: &str) -> FmtResult<()> {
            for c in s.chars() {
                try!(match c {
                    '&' => self.inner.write_str("&amp;"),
                    '<' => self.inner.write_str("&lt;"),
                    '>' => self.inner.write_str("&gt;"),
                    '"' => self.inner.write_str("&quot;"),
                    '\'' => self.inner.write_str("&#39;"),
                    _ => write!(self.inner, "{}", c),
                });
            }
            Ok(())
        }
    }

    #[inline]
    pub fn escape<F>(w: &mut FmtWriter, f: F) -> FmtResult<()> where
        F: FnOnce(&mut FmtWriter) -> FmtResult<()>
    {
        f(&mut Escaper { inner: w })
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
