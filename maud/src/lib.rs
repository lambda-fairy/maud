#![no_std]

//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: https://maud.lambda.xyz/

#![doc(html_root_url = "https://docs.rs/maud/0.22.3")]

extern crate alloc;

use alloc::string::String;
use core::fmt::{self, Write};

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

impl<T: fmt::Display + ?Sized> Render for T {
    fn render_to(&self, w: &mut String) {
        let _ = write!(Escaper::new(w), "{}", self);
    }
}

/// Spicy hack to specialize `Render` for `T: AsRef<str>`.
///
/// The `std::fmt` machinery is rather heavyweight, both in code size and speed.
/// It would be nice to skip this overhead for the common cases of `&str` and
/// `String`. But the obvious solution uses *specialization*, which (as of this
/// writing) requires Nightly. The [*inherent method specialization*][1] trick
/// is less clear but works on Stable.
///
/// This module is an implementation detail and should not be used directly.
///
/// [1]: https://github.com/dtolnay/case-studies/issues/14
#[doc(hidden)]
pub mod render {
    use crate::Render;
    use alloc::string::String;
    use core::fmt::Write;
    use maud_htmlescape::Escaper;

    pub trait RenderInternal {
        fn __maud_render_to(&self, w: &mut String);
    }

    pub struct RenderWrapper<'a, T: ?Sized>(pub &'a T);

    impl<'a, T: AsRef<str> + ?Sized> RenderWrapper<'a, T> {
        pub fn __maud_render_to(&self, w: &mut String) {
            let _ = Escaper::new(w).write_str(self.0.as_ref());
        }
    }

    impl<'a, T: Render + ?Sized> RenderInternal for RenderWrapper<'a, T> {
        fn __maud_render_to(&self, w: &mut String) {
            self.0.render_to(w);
        }
    }
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug, Clone, Copy)]
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

impl<T: AsRef<str> + Into<String>> From<PreEscaped<T>> for String {
    fn from(value: PreEscaped<T>) -> String {
        value.into_string()
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
    extern crate std;

    use crate::PreEscaped;
    use alloc::{boxed::Box, string::String};
    use iron::{
        headers::ContentType,
        modifier::{Modifier, Set},
        modifiers::Header,
        response::{Response, WriteBody},
    };
    use std::io;

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
    extern crate std;

    use crate::PreEscaped;
    use alloc::string::String;
    use rocket::{
        http::ContentType,
        request::Request,
        response::{self, Responder, Response},
    };
    use std::io::Cursor;

    impl<'r, 'o: 'r> Responder<'r, 'o> for PreEscaped<String> {
        fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
            Response::build()
                .header(ContentType::HTML)
                .sized_body(self.0.len(), Cursor::new(self.0))
                .ok()
        }
    }
}

#[cfg(feature = "actix-web")]
mod actix_support {
    use crate::PreEscaped;
    use actix_web_dep::{Error, HttpRequest, HttpResponse, Responder};
    use alloc::string::String;
    use futures_util::future::{ok, Ready};

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

#[cfg(feature = "tide")]
mod tide_support {
    use crate::PreEscaped;
    use alloc::string::String;
    use tide::{http::mime, Response, StatusCode};

    impl From<PreEscaped<String>> for Response {
        fn from(markup: PreEscaped<String>) -> Response {
            Response::builder(StatusCode::Ok)
                .body(markup.into_string())
                .content_type(mime::HTML)
                .build()
        }
    }
}

#[cfg(feature = "axum")]
mod axum_support {
    use crate::PreEscaped;
    use alloc::string::String;
    use axum::{
        body::Body,
        http::{header, HeaderValue, Response, StatusCode},
        response::IntoResponse,
    };

    impl IntoResponse for PreEscaped<String> {
        type Body = Body;
        type BodyError = <Self::Body as axum::body::HttpBody>::Error;

        fn into_response(self) -> Response<Body> {
            let mut res = Response::new(Body::from(self.0));
            *res.status_mut() = StatusCode::OK;
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            res
        }
    }
}
