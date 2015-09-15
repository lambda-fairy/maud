//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: http://lfairy.gitbooks.io/maud/content/

use std::fmt;
use std::io;

/// An adapter that escapes HTML special characters.
///
/// # Example
///
/// ```
/// # use maud::Escaper;
/// use std::fmt::Write;
/// write!(Escaper::new(String::new()), "<script>launchMissiles()</script>").unwrap();
/// assert_eq!(result.into_inner(), "&lt;script&gt;launchMissiles()&lt;/script&gt;");
/// ```
pub struct Escaper<W: fmt::Write> {
    inner: W,
}

impl<W: fmt::Write> Escaper<W> {
    /// Creates an `Escaper` from a `std::fmt::Write`.
    pub fn new(inner: W) -> Escaper<W> {
        Escaper { inner: inner }
    }

    /// Extracts the inner writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: fmt::Write> fmt::Write for Escaper<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            try!(match c {
                '&' => self.inner.write_str("&amp;"),
                '<' => self.inner.write_str("&lt;"),
                '>' => self.inner.write_str("&gt;"),
                '"' => self.inner.write_str("&quot;"),
                '\'' => self.inner.write_str("&#39;"),
                _ => self.inner.write_char(c),
            });
        }
        Ok(())
    }
}

/// Wraps a `std::io::Write` in a `std::fmt::Write`.
///
/// Most I/O libraries work with binary data (`[u8]`), but Maud outputs
/// Unicode strings (`str`). This adapter links them together by
/// encoding the output as UTF-8.
///
/// # Example
///
/// ```rust,ignore
/// use std::io;
/// let mut writer = Utf8Writer::new(io::stdout());
/// let _ = html!(writer, p { "Hello, " $name "!" });
/// let result = writer.into_result();
/// result.unwrap();
/// ```
pub struct Utf8Writer<W: io::Write> {
    inner: W,
    result: io::Result<()>,
}

impl<W: io::Write> Utf8Writer<W> {
    /// Creates a `Utf8Writer` from a `std::io::Write`.
    pub fn new(inner: W) -> Utf8Writer<W> {
        Utf8Writer {
            inner: inner,
            result: Ok(()),
        }
    }

    /// Extracts the inner writer, along with any errors encountered
    /// along the way.
    pub fn into_inner(self) -> (W, io::Result<()>) {
        let Utf8Writer { inner, result } = self;
        (inner, result)
    }

    /// Drops the inner writer, returning any errors encountered
    /// along the way.
    pub fn into_result(self) -> io::Result<()> {
        self.result
    }
}

impl<W: io::Write> fmt::Write for Utf8Writer<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match io::Write::write_all(&mut self.inner, s.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.result = Err(e);
                Err(fmt::Error)
            }
        }
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        match io::Write::write_fmt(&mut self.inner, args) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.result = Err(e);
                Err(fmt::Error)
            }
        }
    }
}
