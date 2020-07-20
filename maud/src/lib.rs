#![cfg_attr(feature = "unstable", feature(min_specialization))]

//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: https://maud.lambda.xyz/

#![doc(html_root_url = "https://docs.rs/maud/0.22.0")]

#[cfg(feature = "actix-web")] extern crate actix_web_dep;
#[cfg(feature = "iron")] extern crate iron;
#[cfg(feature = "rocket")] extern crate rocket;

use std::fmt::{self, Write};

pub use maud_macros::{html, html_debug};

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
/// ```rust
/// # #![feature(proc_macro_hygiene)]
/// use maud::{html, Markup, Render};
///
/// /// Provides a shorthand for linking to a CSS stylesheet.
/// pub struct Stylesheet(&'static str);
///
/// impl Render for Stylesheet {
///     fn render(&self) -> Markup {
///         html! {
///             link rel="stylesheet" type="text/css" href=(self.0);
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

#[cfg(feature = "unstable")]
impl<T: fmt::Display + ?Sized> Render for T {
    default fn render_to(&self, w: &mut String) {
        let _ = write!(Escaper::new(w), "{}", self);
    }
}

impl<T: AsRef<str>> Render for PreEscaped<T> {
    fn render_to(&self, w: &mut String) {
        w.push_str(self.0.as_ref());
    }
}

#[cfg(feature = "unstable")]
impl Render for String {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_str(self);
    }
}

#[cfg(feature = "unstable")]
impl Render for str {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_str(self);
    }
}

#[cfg(not(feature = "unstable"))]
impl<T: fmt::Display + ?Sized> Render for T {
    fn render_to(&self, w: &mut String) {
        let _ = write!(Escaper::new(w), "{}", self);
    }
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug, Clone, Copy)]
pub struct PreEscaped<T: AsRef<str>>(pub T);

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

pub use maud_htmlescape::Escaper;

/// The literal string `<!DOCTYPE html>`.
///
/// # Example
///
/// A minimal web page:
///
/// ```rust
/// # #![feature(proc_macro_hygiene)]
/// use maud::{DOCTYPE, html};
///
/// let markup = html! {
///     (DOCTYPE)
///     html {
///         head {
///             meta charset="utf-8";
///             title { "Test page" }
///         }
///         body {
///             p { "Hello, world!" }
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
    use crate::PreEscaped;

    impl Modifier<Response> for PreEscaped<String> {
        fn modify(self, response: &mut Response) {
            response
                .set_mut(Header(ContentType::html()))
                .set_mut(Box::new(self) as Box<dyn WriteBody>);
        }
    }

    impl WriteBody for PreEscaped<String> {
        fn write_body(&mut self, body: &mut dyn io::Write) -> io::Result<()> {
            self.0.write_body(body)
        }
    }
}

#[cfg(feature = "rocket")]
mod rocket_support {
    use rocket::http::{ContentType, Status};
    use rocket::request::Request;
    use rocket::response::{Responder, Response};
    use std::io::Cursor;
    use crate::PreEscaped;

    impl Responder<'static> for PreEscaped<String> {
        fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
            Response::build()
                .header(ContentType::HTML)
                .sized_body(Cursor::new(self.0))
                .ok()
        }
    }
}

#[cfg(feature = "actix-web")]
mod actix_support {
    use crate::PreEscaped;
    use actix_web_dep::{Responder, HttpResponse, HttpRequest, Error};
    use futures::future::{ok, Ready};

    impl Responder for PreEscaped<String> {
        type Error = Error;
        type Future = Ready<Result<HttpResponse, Self::Error>>;
        fn respond_to(self, _req: &HttpRequest) -> Self::Future {
            ok(HttpResponse::Ok()
               .content_type("text/html; charset=utf-8")
               .body(self.0))
        }
    }
}
