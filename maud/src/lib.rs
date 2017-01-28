#![feature(specialization)]

//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: https://maud.lambda.xyz/

#[cfg(feature = "iron")] extern crate iron;
#[cfg(feature = "rocket")] extern crate rocket;

use std::fmt::{self, Write};

/// Represents a type that can be rendered as HTML.
///
/// If your type implements [`Display`][1], then it will implement this
/// trait automatically through a blanket impl.
///
/// [1]: https://doc.rust-lang.org/std/fmt/trait.Display.html
///
/// On the other hand, if your type has a custom HTML representation,
/// then you can implement `Render` by hand. To do this, override
/// either the `.render()` or `.render_to()` methods; since each is
/// defined in terms of the other, you only need to implement one of
/// them. See the example below.
///
/// # Minimal implementation
///
/// An implementation of this trait must override at least one of
/// `.render()` or `.render_to()`. Since the default definitions of
/// these methods call each other, not doing this will result in
/// infinite recursion.
///
/// # Example
///
/// ```rust,ignore
/// /// Provides a shorthand for linking to a CSS stylesheet.
/// pub struct Stylesheet(&'static str);
///
/// impl Render for Stylesheet {
///     fn render(&self) -> Markup {
///         html! {
///             link rel="stylesheet" type="text/css" href=(self.0) /
///         }
///     }
/// }
/// ```
pub trait Render {
    /// Renders `self` as a block of `Markup`.
    fn render(&self) -> Markup {
        let mut buffer = String::new();
        self.render_to(&mut buffer);
        PreEscaped(buffer)
    }

    /// Appends a representation of `self` to the given buffer.
    ///
    /// Its default implementation just calls `.render()`, but you may
    /// override it with something more efficient.
    ///
    /// Note that no further escaping is performed on data written to
    /// the buffer. If you override this method, you must make sure that
    /// any data written is properly escaped, whether by hand or using
    /// the [`Escaper`](struct.Escaper.html) wrapper struct.
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.render().into_string());
    }
}

impl<T: fmt::Display + ?Sized> Render for T {
    default fn render_to(&self, w: &mut String) {
        let _ = write!(Escaper::new(w), "{}", self);
    }
}

impl Render for String {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_str(self);
    }
}

impl Render for str {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_str(self);
    }
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug)]
pub struct PreEscaped<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for PreEscaped<T> {
    fn render_to(&self, w: &mut String) {
        w.push_str(self.0.as_ref());
    }
}

/// A block of markup is a string that does not need to be escaped.
///
/// The `html!` macro expands to an expression of this type.
pub type Markup = PreEscaped<String>;

impl<T: AsRef<str> + Into<String>> PreEscaped<T> {
    /// Converts the inner value to a string.
    pub fn into_string(self) -> String {
        self.0.into()
    }
}

impl<T: AsRef<str> + Into<String>> Into<String> for PreEscaped<T> {
    fn into(self) -> String {
        self.into_string()
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
///
/// All other characters are passed through unchanged.
///
/// **Note:** In versions prior to 0.13, the single quote (`'`) was
/// escaped as well.
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
                _ => unsafe { self.0.as_mut_vec().push(b) },
            }
        }
        Ok(())
    }
}

/// The literal string `<!DOCTYPE html>`.
///
/// # Example
///
/// A minimal web page:
///
/// ```rust,ignore
/// use maud::DOCTYPE;
///
/// let markup = html! {
///     (DOCTYPE)
///     html {
///         head {
///             meta charset="utf-8" /
///             title "Test page"
///         }
///         body {
///             p "Hello, world!"
///         }
///     }
/// };
/// ```
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");

#[cfg(feature = "iron")]
mod iron_support {
    use std::io;
    use iron::headers::ContentType;
    use iron::modifier::{Modifier, Set};
    use iron::modifiers::Header;
    use iron::response::{Response, WriteBody};
    use PreEscaped;

    impl Modifier<Response> for PreEscaped<String> {
        fn modify(self, response: &mut Response) {
            response
                .set_mut(Header(ContentType::html()))
                .set_mut(Box::new(self) as Box<WriteBody>);
        }
    }

    impl WriteBody for PreEscaped<String> {
        fn write_body(&mut self, body: &mut io::Write) -> io::Result<()> {
            self.0.write_body(body)
        }
    }
}

#[cfg(feature = "rocket")]
mod rocket_support {
    use rocket::http::{ContentType, Status};
    use rocket::response::{Responder, Response};
    use std::io::Cursor;
    use PreEscaped;

    impl Responder<'static> for PreEscaped<String> {
        fn respond(self) -> Result<Response<'static>, Status> {
            Response::build()
                .header(ContentType::HTML)
                .sized_body(Cursor::new(self.0))
                .ok()
        }
    }
}
