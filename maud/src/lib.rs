//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: http://lfairy.gitbooks.io/maud/content/

use std::fmt;
use std::io;

/// Escapes an HTML value.
pub fn escape(s: &str) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    rt::Escaper { inner: &mut buf }.write_str(s).unwrap();
    buf
}

/// A block of HTML markup, as returned by the `html!` macro.
///
/// Use `.to_string()` to convert it to a `String`, or `.render()` to
/// write it directly to a handle.
pub struct Markup<F> {
    callback: F,
}

impl<F> Markup<F> where F: Fn(&mut fmt::Write) -> fmt::Result {
    /// Renders the markup to a `std::io::Write`.
    pub fn render<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        struct Adaptor<'a, W: ?Sized + 'a> {
            inner: &'a mut W,
            error: io::Result<()>,
        }

        impl<'a, W: ?Sized + io::Write> fmt::Write for Adaptor<'a, W> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(fmt::Error)
                    },
                }
            }
        }

        let mut output = Adaptor { inner: w, error: Ok(()) };
        match self.render_fmt(&mut output) {
            Ok(()) => Ok(()),
            Err(_) => output.error,
        }
    }

    /// Renders the markup to a `std::fmt::Write`.
    pub fn render_fmt(&self, w: &mut fmt::Write) -> fmt::Result {
        (self.callback)(w)
    }
}

impl<F> ToString for Markup<F> where F: Fn(&mut fmt::Write) -> fmt::Result {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        self.render_fmt(&mut buf).unwrap();
        buf
    }
}

/// Internal functions used by the `maud_macros` package. You should
/// never need to call these directly.
#[doc(hidden)]
pub mod rt {
    use std::fmt;
    use super::Markup;

    #[inline]
    pub fn make_markup<F>(f: F) -> Markup<F>
        where F: Fn(&mut fmt::Write) -> fmt::Result
    {
        Markup { callback: f }
    }

    /// rustc is a butt and doesn't let us quote macro invocations
    /// directly. So we factor the `write!` call into a separate
    /// function and use that instead.
    ///
    /// See <https://github.com/rust-lang/rust/issues/16617>
    #[inline]
    pub fn write_fmt<T: fmt::Display>(w: &mut fmt::Write, value: T) -> fmt::Result {
        write!(w, "{}", value)
    }

    pub struct Escaper<'a, 'b: 'a> {
        pub inner: &'a mut (fmt::Write + 'b),
    }

    impl<'a, 'b> fmt::Write for Escaper<'a, 'b> {
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
