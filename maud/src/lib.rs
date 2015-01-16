//! Super fast HTML template engine.

#![allow(unstable)]

use std::fmt;
use std::io::{IoError, IoErrorKind, IoResult};

/// Escape an HTML value.
pub fn escape(s: &str) -> String {
    let mut buf = String::new();
    rt::escape(&mut buf, |w| w.write_str(s)).unwrap();
    buf
}

/// A block of HTML markup, as returned by the `html!` macro.
pub struct Markup<'a> {
    callback: &'a (Fn(&mut fmt::Writer) -> fmt::Result + 'a),
}

impl<'a> Markup<'a> {
    /// Render the markup to a `String`.
    pub fn render(&self) -> String {
        let mut buf = String::new();
        self.render_fmt(&mut buf).unwrap();
        buf
    }

    /// Render the markup to a `std::io::Writer`.
    pub fn render_to(&self, w: &mut Writer) -> IoResult<()> {
        struct WriterWrapper<'a, 'b: 'a> {
            inner: &'a mut (Writer + 'b),
        }
        impl<'a, 'b> fmt::Writer for WriterWrapper<'a, 'b> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.inner.write_str(s).map_err(|_| fmt::Error)
            }
        }
        self.render_fmt(&mut WriterWrapper { inner: w })
            .map_err(|_| IoError {
                kind: IoErrorKind::OtherIoError,
                desc: "formatting error",
                detail: None,
            })
    }

    /// Render the markup to a `std::fmt::Writer`.
    pub fn render_fmt(&self, w: &mut fmt::Writer) -> fmt::Result {
        (self.callback)(w)
    }
}

/// Internal functions used by the `maud_macros` package. You should
/// never need to call these directly.
#[doc(hidden)]
pub mod rt {
    use std::fmt;
    use super::Markup;

    #[inline]
    pub fn make_markup<'a>(f: &'a (Fn(&mut fmt::Writer) -> fmt::Result + 'a)) -> Markup<'a> {
        Markup { callback: f }
    }

    struct Escaper<'a, 'b: 'a> {
        inner: &'a mut (fmt::Writer + 'b),
    }

    impl<'a, 'b> fmt::Writer for Escaper<'a, 'b> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
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
    pub fn escape<F>(w: &mut fmt::Writer, f: F) -> fmt::Result where
        F: FnOnce(&mut fmt::Writer) -> fmt::Result
    {
        f(&mut Escaper { inner: w })
    }
}
