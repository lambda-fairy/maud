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
    /// Renders `self` to the given writer.
    ///
    /// Note that the writer does *not* perform automatic escaping. You
    /// must make sure that any data written is properly escaped,
    /// whether by hand or using the `Escaper` wrapper struct.
    fn render(&self, &mut fmt::Write) -> fmt::Result;
}

impl<T: fmt::Display + ?Sized> Render for T {
    default fn render(&self, w: &mut fmt::Write) -> fmt::Result {
        write!(Escaper::new(w), "{}", self)
    }
}

impl Render for String {
    fn render(&self, w: &mut fmt::Write) -> fmt::Result {
        Escaper::new(w).write_str(self)
    }
}

impl Render for str {
    fn render(&self, w: &mut fmt::Write) -> fmt::Result {
        Escaper::new(w).write_str(self)
    }
}

/// Represents a type that can be rendered as HTML, where the rendering
/// operation must consume the input data.
pub trait RenderOnce {
    fn render_once(self, &mut fmt::Write) -> fmt::Result;
}

impl<'a, T: Render + ?Sized> RenderOnce for &'a T {
    /// Renders `self` to the given writer.
    ///
    /// Note that the writer does *not* perform automatic escaping. You
    /// must make sure that any data written is properly escaped,
    /// whether by hand or using the `Escaper` wrapper struct.
    fn render_once(self, w: &mut fmt::Write) -> fmt::Result {
        Render::render(self, w)
    }
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug)]
pub struct PreEscaped<T>(pub T);

impl<T: fmt::Display> Render for PreEscaped<T> {
    default fn render(&self, w: &mut fmt::Write) -> fmt::Result {
        write!(w, "{}", self.0)
    }
}

impl Render for PreEscaped<String> {
    fn render(&self, w: &mut fmt::Write) -> fmt::Result {
        w.write_str(&self.0)
    }
}

impl<'a> Render for PreEscaped<&'a str> {
    fn render(&self, w: &mut fmt::Write) -> fmt::Result {
        w.write_str(self.0)
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
/// let mut escaper = Escaper::new(String::new());
/// write!(escaper, "<script>launchMissiles()</script>").unwrap();
/// assert_eq!(escaper.into_inner(), "&lt;script&gt;launchMissiles()&lt;/script&gt;");
/// ```
pub struct Escaper<W> {
    inner: W,
}

impl<W> Escaper<W> {
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
