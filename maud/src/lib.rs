//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: http://lfairy.gitbooks.io/maud/content/

/// Escapes an HTML value.
pub fn escape(s: &str) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    rt::Escaper { inner: &mut buf }.write_str(s).unwrap();
    buf
}

/// Internal functions used by the `maud_macros` package. You should
/// never need to call these directly.
#[doc(hidden)]
pub mod rt {
    use std::fmt;

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
