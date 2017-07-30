//! Internal support code used by the [Maud] template engine.
//!
//! You should not need to depend on this crate directly.
//!
//! [Maud]: https://maud.lambda.xyz

#![doc(html_root_url = "https://docs.rs/maud_htmlescape/0.17.0")]

use std::fmt;

/// An adapter that escapes HTML special characters.
///
/// The following characters are escaped:
///
/// * `&` is escaped as `&amp;`
/// * `<` is escaped as `&lt;`
/// * `>` is escaped as `&gt;`
/// * `"` is escaped as `&quot;`
///
/// All other characters are passed through unchanged.
///
/// **Note:** In versions prior to 0.13, the single quote (`'`) was
/// escaped as well.
///
/// # Example
///
/// ```rust,ignore
/// use std::fmt::Write;
/// let mut s = String::new();
/// write!(Escaper::new(&mut s), "<script>launchMissiles()</script>").unwrap();
/// assert_eq!(s, "&lt;script&gt;launchMissiles()&lt;/script&gt;");
/// ```
pub struct Escaper<'a>(&'a mut String);

impl<'a> Escaper<'a> {
    /// Creates an `Escaper` from a `String`.
    pub fn new(buffer: &'a mut String) -> Escaper<'a> {
        Escaper(buffer)
    }
}

impl<'a> fmt::Write for Escaper<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            match b {
                b'&' => self.0.push_str("&amp;"),
                b'<' => self.0.push_str("&lt;"),
                b'>' => self.0.push_str("&gt;"),
                b'"' => self.0.push_str("&quot;"),
                _ => unsafe { self.0.as_mut_vec().push(b) },
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Write;
    use Escaper;

    #[test]
    fn it_works() {
        let mut s = String::new();
        write!(Escaper::new(&mut s), "<script>launchMissiles()</script>").unwrap();
        assert_eq!(s, "&lt;script&gt;launchMissiles()&lt;/script&gt;");
    }
}
