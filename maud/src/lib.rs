#![no_std]

//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: https://maud.lambda.xyz/

#![doc(html_root_url = "https://docs.rs/maud/0.23.0")]

extern crate alloc;

use alloc::{borrow::Cow, boxed::Box, string::String};
use core::fmt::{self, Arguments, Write};

pub use maud_macros::{html, html_debug};

mod escape;

/// Represents a type that can be rendered as HTML.
///
/// To implement this for your own type, override either the `.html()`
/// or `.to_html()` methods; since each is defined in terms of the
/// other, you only need to implement one of them. See the example below.
///
/// # Minimal implementation
///
/// An implementation of this trait must override at least one of
/// `.html()` or `.to_html()`. Since the default definitions of these
/// methods call each other, not doing this will result in infinite
/// recursion.
///
/// # Example
///
/// ```rust
/// use maud::{html, Html, ToHtml};
///
/// /// Provides a shorthand for linking to a CSS stylesheet.
/// pub struct Stylesheet(&'static str);
///
/// impl ToHtml for Stylesheet {
///     fn to_html(&self) -> Html {
///         html! {
///             link rel="stylesheet" type="text/css" href=(self.0);
///         }
///     }
/// }
/// ```
pub trait ToHtml {
    /// Creates an HTML representation of `self`.
    fn to_html(&self) -> Html {
        let mut buffer = Html::default();
        self.html(&mut buffer);
        buffer
    }

    /// Appends an HTML representation of `self` to the given buffer.
    ///
    /// Its default implementation just calls `.to_html()`, but you may
    /// override it with something more efficient.
    fn html(&self, buffer: &mut Html) {
        self.to_html().html(buffer)
    }
}

impl ToHtml for str {
    fn html(&self, buffer: &mut Html) {
        // XSS-Safety: Special characters will be escaped by `escape_to_string`.
        escape::escape_to_string(self, buffer.as_mut_string_unchecked());
    }
}

impl ToHtml for String {
    fn html(&self, buffer: &mut Html) {
        str::html(self, buffer);
    }
}

impl<'a> ToHtml for Cow<'a, str> {
    fn html(&self, buffer: &mut Html) {
        str::html(self, buffer);
    }
}

impl<'a> ToHtml for Arguments<'a> {
    fn html(&self, buffer: &mut Html) {
        let _ = buffer.write_fmt(*self);
    }
}

impl<'a, T: ToHtml + ?Sized> ToHtml for &'a T {
    fn html(&self, buffer: &mut Html) {
        T::html(self, buffer);
    }
}

impl<'a, T: ToHtml + ?Sized> ToHtml for &'a mut T {
    fn html(&self, buffer: &mut Html) {
        T::html(self, buffer);
    }
}

impl<T: ToHtml + ?Sized> ToHtml for Box<T> {
    fn html(&self, buffer: &mut Html) {
        T::html(self, buffer);
    }
}

macro_rules! impl_render_with_display {
    ($($ty:ty)*) => {
        $(
            impl ToHtml for $ty {
                fn html(&self, buffer: &mut Html) {
                    let _ = write!(buffer, "{self}");
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
            impl ToHtml for $ty {
                fn html(&self, buffer: &mut Html) {
                    // XSS-Safety: The characters '0' through '9', and '-', are HTML safe.
                    let _ = itoa::fmt(buffer.as_mut_string_unchecked(), *self);
                }
            }
        )*
    };
}

impl_render_with_itoa! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
}

/// A fragment of HTML.
///
/// This is the type that's returned by the [`html!`] macro.
#[derive(Clone, Debug, Default)]
pub struct Html {
    inner: Cow<'static, str>,
}

impl Html {
    /// Creates an HTML fragment from a constant string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use maud::Html;
    ///
    /// let analytics_script = Html::from_const("<script>trackThisPageView();</script>");
    /// ```
    ///
    /// # Security
    ///
    /// The given string must be a *compile-time constant*: either a
    /// literal, or a reference to a `const` value. This ensures that
    /// the string is as trustworthy as the code itself.
    ///
    /// If the string is not a compile-time constant, use
    /// [`Html::from_unchecked`] instead, and document why the call is
    /// safe.
    ///
    /// In the future, when [`const` string parameters] are available on
    /// Rust stable, this rule will be enforced by the API.
    ///
    /// [`const` string parameters]: https://blog.rust-lang.org/inside-rust/2021/09/06/Splitting-const-generics.html#featureadt_const_params
    pub const fn from_const(html_string: &'static str) -> Self {
        Html {
            inner: Cow::Borrowed(html_string),
        }
    }

    #[cfg(feature = "sanitize")]
    /// Takes an untrusted HTML fragment and makes it safe.
    ///
    /// # Example
    ///
    /// ```rust
    /// use maud::Html;
    ///
    /// let untrusted_html = "<p><script>alert('bwahaha!');</script></p>";
    ///
    /// let clean_html = Html::sanitize(untrusted_html);
    ///
    /// assert_eq!(clean_html.into_string(), "<p></p>");
    /// ```
    pub fn sanitize(untrusted_html_string: &str) -> Self {
        // XSS-Safety: Ammonia sanitizes the input.
        Self::from_unchecked(ammonia::clean(untrusted_html_string))
    }

    /// Creates an HTML fragment from a string, without escaping it.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn load_header_from_config() -> String { String::new() }
    /// use maud::Html;
    ///
    /// // XSS-Safety: The config can only be edited by an admin.
    /// let header = Html::from_unchecked(load_header_from_config());
    /// ```
    ///
    /// # Security
    ///
    /// It is your responsibility to ensure that the string comes from a
    /// trusted source. Misuse of this function can lead to [cross-site
    /// scripting attacks (XSS)][xss].
    ///
    /// It is strongly recommended to include a `// XSS-Safety:` comment
    /// that explains why this call is safe.
    ///
    /// If your organization has a security team, consider asking them
    /// for review.
    ///
    /// [xss]: https://www.cloudflare.com/en-au/learning/security/threats/cross-site-scripting/
    pub fn from_unchecked(html_string: impl Into<Cow<'static, str>>) -> Self {
        Self {
            inner: html_string.into(),
        }
    }

    /// For internal use only.
    #[doc(hidden)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Cow::Owned(String::with_capacity(capacity)),
        }
    }

    /// Appends the HTML representation of the given value to `self`.
    pub fn push(&mut self, value: &(impl ToHtml + ?Sized)) {
        value.html(self);
    }

    /// Exposes the underlying buffer as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Exposes the underlying buffer as a `&mut String`.
    ///
    /// # Security
    ///
    /// As with [`Html::from_unchecked`], it is your responsibility to
    /// ensure that any additions are properly escaped.
    ///
    /// It is strongly recommended to include a `// XSS-Safety:` comment
    /// that explains why this call is safe.
    ///
    /// If your organization has a security team, consider asking them
    /// for review.
    pub fn as_mut_string_unchecked(&mut self) -> &mut String {
        self.inner.to_mut()
    }

    /// Converts the inner value to a `String`.
    pub fn into_string(self) -> String {
        self.inner.into_owned()
    }
}

impl Write for Html {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.push(text);
        Ok(())
    }
}

impl ToHtml for Html {
    fn html(&self, buffer: &mut Html) {
        // XSS-Safety: `self` is already guaranteed to be trusted HTML.
        buffer.as_mut_string_unchecked().push_str(self.as_str());
    }
}

impl From<Html> for String {
    fn from(html: Html) -> String {
        html.into_string()
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
/// let page = html! {
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
pub const DOCTYPE: Html = Html::from_const("<!DOCTYPE html>");

#[cfg(feature = "rocket")]
mod rocket_support {
    extern crate std;

    use crate::Html;
    use rocket::{
        http::{ContentType, Status},
        request::Request,
        response::{Responder, Response},
    };
    use std::io::Cursor;

    impl Responder<'static> for Html {
        fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
            Response::build()
                .header(ContentType::HTML)
                .sized_body(Cursor::new(self.into_string()))
                .ok()
        }
    }
}

#[cfg(feature = "actix-web")]
mod actix_support {
    use crate::Html;
    use actix_web_dep::{Error, HttpRequest, HttpResponse, Responder};
    use futures_util::future::{ok, Ready};

    impl Responder for Html {
        type Error = Error;
        type Future = Ready<Result<HttpResponse, Self::Error>>;
        fn respond_to(self, _req: &HttpRequest) -> Self::Future {
            ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(self.into_string()))
        }
    }
}

#[cfg(feature = "tide")]
mod tide_support {
    use crate::Html;
    use tide::{http::mime, Response, StatusCode};

    impl From<Html> for Response {
        fn from(html: Html) -> Response {
            Response::builder(StatusCode::Ok)
                .body(html.into_string())
                .content_type(mime::HTML)
                .build()
        }
    }
}

#[cfg(feature = "axum")]
mod axum_support {
    use crate::Html;
    use axum_core::{body::BoxBody, response::IntoResponse};
    use http::{header, HeaderMap, HeaderValue, Response};

    impl IntoResponse for Html {
        fn into_response(self) -> Response<BoxBody> {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            (headers, self.inner).into_response()
        }
    }
}
