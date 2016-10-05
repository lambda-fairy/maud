#![feature(specialization)]

//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: https://maud.lambda.xyz/

#[cfg(feature = "iron")] extern crate iron;

use std::fmt::{self, Write};

/// Represents a type that can be rendered as HTML.
///
/// Most of the time you should implement `std::fmt::Display` instead,
/// which will be picked up by the blanket impl.
pub trait Render {
    /// Appends a representation of `self` to the given string.
    ///
    /// Note that the runtime does *not* perform automatic escaping. You
    /// must make sure that any data written is properly escaped,
    /// whether by hand or using the `Escaper` wrapper struct.
    fn render(&self, &mut String);
}

impl<T: fmt::Display + ?Sized> Render for T {
    default fn render(&self, w: &mut String) {
        write!(Escaper::new(w), "{}", self).unwrap();
    }
}

impl Render for String {
    fn render(&self, w: &mut String) {
        Escaper::new(w).write_str(self).unwrap();
    }
}

impl Render for str {
    fn render(&self, w: &mut String) {
        Escaper::new(w).write_str(self).unwrap();
    }
}

/// Represents a type that can be rendered as HTML, where the rendering
/// operation must consume the value.
pub trait RenderOnce {
    /// Appends a representation of `self` to the given string.
    ///
    /// Note that the runtime does *not* perform automatic escaping. You
    /// must make sure that any data written is properly escaped,
    /// whether by hand or using the `Escaper` wrapper struct.
    fn render_once(self, &mut String);
}

impl<'a, T: Render + ?Sized> RenderOnce for &'a T {
    fn render_once(self, w: &mut String) {
        Render::render(self, w);
    }
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug)]
pub struct PreEscaped<T>(pub T);

impl<T: fmt::Display> Render for PreEscaped<T> {
    default fn render(&self, w: &mut String) {
        write!(w, "{}", self.0).unwrap();
    }
}

impl Render for PreEscaped<String> {
    fn render(&self, w: &mut String) {
        w.push_str(&self.0);
    }
}

impl<'a> Render for PreEscaped<&'a str> {
    fn render(&self, w: &mut String) {
        w.push_str(self.0);
    }
}

/// A block of markup is a string that does not need to be escaped.
///
/// The `html!` macro expands to an expression of this type.
pub type Markup = PreEscaped<String>;

impl PreEscaped<String> {
    /// Extracts the inner `String`. This is a synonym for `self.0`.
    pub fn into_string(self) -> String {
        self.0
    }
}

/// An adapter that escapes HTML special characters.
///
/// The following characters are escaped:
///
/// * `&` is escaped as `&amp;`
/// * `<` is escaped as `&lt;`
/// * `>` is escaped as `&gt;`
/// * `"` is escaped as `&quot;`
/// * `'` is escaped as `&#39;`
///
/// All other characters are passed through unchanged.
///
/// # Example
///
/// ```
/// # use maud::Escaper;
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
                b'\'' => self.0.push_str("&#39;"),
                _ => unsafe { self.0.as_mut_vec().push(b) },
            }
        }
        Ok(())
    }
}

#[cfg(feature = "iron")]
mod iron_support {
    use std::io;
    use iron::headers::ContentType;
    use iron::modifier::{Modifier, Set};
    use iron::modifiers::Header;
    use iron::response::{Response, ResponseBody, WriteBody};
    use PreEscaped;

    impl Modifier<Response> for PreEscaped<String> {
        fn modify(self, response: &mut Response) {
            response
                .set_mut(Header(ContentType::html()))
                .set_mut(Box::new(self) as Box<WriteBody>);
        }
    }

    impl WriteBody for PreEscaped<String> {
        fn write_body(&mut self, body: &mut ResponseBody) -> io::Result<()> {
            self.0.write_body(body)
        }
    }
}
