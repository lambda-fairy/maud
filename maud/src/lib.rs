#![cfg_attr(not(feature = "hotreload"), no_std)]

//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: https://maud.lambda.xyz/

#![doc(html_root_url = "https://docs.rs/maud/0.26.0")]

extern crate alloc;

use alloc::{borrow::Cow, boxed::Box, string::String, sync::Arc};
use core::fmt::{self, Arguments, Display, Write};

pub use maud_macros::html as html_static;

#[cfg(not(feature = "hotreload"))]
pub use maud_macros::html;

#[cfg(feature = "hotreload")]
pub use {maud_macros::html_hotreload, maud_macros::html_hotreload as html};

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
/// ```rust
/// use maud::Escaper;
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

impl fmt::Write for Escaper<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        maud_macros_impl::escape_to_string(s, self.0);
        Ok(())
    }
}

/// Represents a type that can be rendered as HTML.
///
/// To implement this for your own type, override either the `.render()`
/// or `.render_to()` methods; since each is defined in terms of the
/// other, you only need to implement one of them. See the example below.
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

impl Render for str {
    fn render_to(&self, w: &mut String) {
        maud_macros_impl::escape_to_string(self, w);
    }
}

impl Render for String {
    fn render_to(&self, w: &mut String) {
        str::render_to(self, w);
    }
}

impl Render for Cow<'_, str> {
    fn render_to(&self, w: &mut String) {
        str::render_to(self, w);
    }
}

impl Render for Arguments<'_> {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_fmt(*self);
    }
}

impl<T: Render + ?Sized> Render for &T {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<T: Render + ?Sized> Render for &mut T {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<T: Render + ?Sized> Render for Box<T> {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

impl<T: Render + ?Sized> Render for Arc<T> {
    fn render_to(&self, w: &mut String) {
        T::render_to(self, w);
    }
}

macro_rules! impl_render_with_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render_to(&self, w: &mut String) {
                    // TODO: remove the explicit arg when Rust 1.58 is released
                    format_args!("{self}", self = self).render_to(w);
                }
            }
        )*
    };
}

impl_render_with_display! {
    char f32 f64
}

macro_rules! impl_render_with_itoa {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render_to(&self, w: &mut String) {
                    w.push_str(itoa::Buffer::new().format(*self));
                }
            }
        )*
    };
}

impl_render_with_itoa! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
}

/// Renders a value using its [`Display`] impl.
///
/// # Example
///
/// ```rust
/// use maud::html;
/// use std::net::Ipv4Addr;
///
/// let ip_address = Ipv4Addr::new(127, 0, 0, 1);
///
/// let markup = html! {
///     "My IP address is: "
///     (maud::display(ip_address))
/// };
///
/// assert_eq!(markup.into_string(), "My IP address is: 127.0.0.1");
/// ```
pub fn display(value: impl Display) -> impl Render {
    struct DisplayWrapper<T>(T);

    impl<T: Display> Render for DisplayWrapper<T> {
        fn render_to(&self, w: &mut String) {
            format_args!("{0}", self.0).render_to(w);
        }
    }

    DisplayWrapper(value)
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug, Clone, Copy)]
pub struct PreEscaped<T>(pub T);

impl<T: AsRef<str>> Render for PreEscaped<T> {
    fn render_to(&self, w: &mut String) {
        w.push_str(self.0.as_ref());
    }
}

/// A block of markup is a string that does not need to be escaped.
///
/// The `html!` macro expands to an expression of this type.
pub type Markup = PreEscaped<String>;

impl<T: Into<String>> PreEscaped<T> {
    /// Converts the inner value to a string.
    pub fn into_string(self) -> String {
        self.0.into()
    }
}

impl<T: Into<String>> From<PreEscaped<T>> for String {
    fn from(value: PreEscaped<T>) -> String {
        value.into_string()
    }
}

impl<T: Default> Default for PreEscaped<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

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

#[cfg(feature = "rocket")]
mod rocket_support {
    extern crate std;

    use crate::PreEscaped;
    use alloc::string::String;
    use rocket::{
        http::ContentType,
        request::Request,
        response::{Responder, Response},
    };
    use std::io::Cursor;

    impl Responder<'_, 'static> for PreEscaped<String> {
        fn respond_to(self, _: &Request) -> rocket::response::Result<'static> {
            Response::build()
                .header(ContentType::HTML)
                .sized_body(self.0.len(), Cursor::new(self.0))
                .ok()
        }
    }
}

#[cfg(feature = "actix-web")]
mod actix_support {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };

    use crate::PreEscaped;
    use actix_web_dep::{
        body::{BodySize, MessageBody},
        http::header,
        web::Bytes,
        HttpRequest, HttpResponse, Responder,
    };
    use alloc::string::String;

    impl MessageBody for PreEscaped<String> {
        type Error = <String as MessageBody>::Error;

        fn size(&self) -> BodySize {
            self.0.size()
        }

        fn poll_next(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Bytes, Self::Error>>> {
            Pin::new(&mut self.0).poll_next(cx)
        }
    }

    impl Responder for PreEscaped<String> {
        type Body = String;

        fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
            HttpResponse::Ok()
                .content_type(header::ContentType::html())
                .message_body(self.0)
                .unwrap()
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
    use axum_core::response::{IntoResponse, Response};
    use http::{header, HeaderMap, HeaderValue};

    impl IntoResponse for PreEscaped<String> {
        fn into_response(self) -> Response {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            (headers, self.0).into_response()
        }
    }
}

#[cfg(feature = "warp")]
mod warp_support {
    use crate::PreEscaped;
    use alloc::string::String;
    use warp::reply::{self, Reply, Response};

    impl Reply for PreEscaped<String> {
        fn into_response(self) -> Response {
            reply::html(self.into_string()).into_response()
        }
    }
}

#[cfg(feature = "submillisecond")]
mod submillisecond_support {
    use crate::PreEscaped;
    use alloc::string::String;
    use submillisecond::{
        http::{header, HeaderMap, HeaderValue},
        response::{IntoResponse, Response},
    };

    impl IntoResponse for PreEscaped<String> {
        fn into_response(self) -> Response {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            (headers, self.0).into_response()
        }
    }
}

#[doc(hidden)]
pub mod macro_private {
    use crate::{display, Render};
    pub use alloc::string::String;
    use core::fmt::Display;
    #[cfg(feature = "hotreload")]
    pub use std::collections::HashMap;
    pub use std::env::var as env_var;

    #[doc(hidden)]
    #[macro_export]
    macro_rules! render_to {
        ($x:expr, $buffer:expr) => {{
            use $crate::macro_private::*;
            match ChooseRenderOrDisplay($x) {
                x => (&&x).implements_render_or_display().render_to(x.0, $buffer),
            }
        }};
    }

    pub use render_to;

    pub struct ChooseRenderOrDisplay<T>(pub T);

    pub struct ViaRenderTag;
    pub struct ViaDisplayTag;

    pub trait ViaRender {
        fn implements_render_or_display(&self) -> ViaRenderTag {
            ViaRenderTag
        }
    }
    pub trait ViaDisplay {
        fn implements_render_or_display(&self) -> ViaDisplayTag {
            ViaDisplayTag
        }
    }

    impl<T: Render> ViaRender for &ChooseRenderOrDisplay<T> {}
    impl<T: Display> ViaDisplay for ChooseRenderOrDisplay<T> {}

    impl ViaRenderTag {
        pub fn render_to<T: Render + ?Sized>(self, value: &T, buffer: &mut String) {
            value.render_to(buffer);
        }
    }

    impl ViaDisplayTag {
        pub fn render_to<T: Display + ?Sized>(self, value: &T, buffer: &mut String) {
            display(value).render_to(buffer);
        }
    }

    #[cfg(feature = "hotreload")]
    pub use maud_macros_impl::*;

    #[cfg(feature = "hotreload")]
    pub fn render_runtime_error(input: &str, e: &str) -> crate::Markup {
        // TODO: print to stdout as well?
        crate::PreEscaped(alloc::format!("\"> --> <div style='background: black; color: white; position: absolute; top: 0; left: 0; z-index: 1000'><h1 style='red'>Template Errors:</h1><pre>{}</pre><br>input:<pre>{}</pre></div>", e, input))
    }
}
