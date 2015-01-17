//! Super fast HTML template engine.

#![allow(unstable)]

use std::fmt;
use std::io::{IoError, IoErrorKind, IoResult};

/// Escape an HTML value.
pub fn escape(s: &str) -> String {
    use std::fmt::Writer;
    let mut buf = String::new();
    rt::Escaper { inner: &mut buf }.write_str(s).unwrap();
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

    /// rustc is a butt and doesn't let us quote macro invocations
    /// directly. So we factor the `write!` call into a separate
    /// function and use that instead.
    ///
    /// See <https://github.com/rust-lang/rust/issues/16617>
    #[inline]
    pub fn write_fmt<T: fmt::String>(w: &mut fmt::Writer, value: T) -> fmt::Result {
        write!(w, "{}", value)
    }

    pub struct Escaper<'a, 'b: 'a> {
        pub inner: &'a mut (fmt::Writer + 'b),
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
}
