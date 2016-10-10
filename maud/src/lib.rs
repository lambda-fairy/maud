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

/// Represents a type that can be rendered as HTML, where the rendering
/// operation consumes the value.
///
/// See the [`Render`](trait.Render.html) documentation for advice on
/// how to use this trait.
pub trait RenderOnce: Sized {
    /// Renders `self` as a block of `Markup`, consuming it in the
    /// process.
    fn render_once(self) -> Markup {
        let mut buffer = String::new();
        self.render_once_to(&mut buffer);
        PreEscaped(buffer)
    }

    /// Appends a representation of `self` to the given string,
    /// consuming `self` in the process.
    ///
    /// Its default implementation just calls `.render_once()`, but you
    /// may override it with something more efficient.
    ///
    /// Note that no further escaping is performed on data written to
    /// the buffer. If you override this method, you must make sure that
    /// any data written is properly escaped, whether by hand or using
    /// the [`Escaper`](struct.Escaper.html) wrapper struct.
    fn render_once_to(self, buffer: &mut String) {
        buffer.push_str(&self.render_once().into_string());
    }
}

impl<'a, T: Render + ?Sized> RenderOnce for &'a T {
    fn render_once_to(self, w: &mut String) {
        self.render_to(w);
    }
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug)]
pub struct PreEscaped<T>(pub T);

impl<T: fmt::Display> Render for PreEscaped<T> {
    default fn render_to(&self, w: &mut String) {
        let _ = write!(w, "{}", self.0);
    }
}

impl Render for PreEscaped<String> {
    fn render_to(&self, w: &mut String) {
        w.push_str(&self.0);
    }
}

impl<'a> Render for PreEscaped<&'a str> {
    fn render_to(&self, w: &mut String) {
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
