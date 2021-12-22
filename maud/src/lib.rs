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
/// # Minimal implementation
///
/// An implementation of this trait must override at least one of
/// `.to_html()` or `.push_html_to()`. Since the default definitions of
/// these methods call each other, not doing this will result in infinite
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
        let mut builder = HtmlBuilder::new();
        self.push_html_to(&mut builder);
        builder.finalize()
    }

    /// Appends an HTML representation of `self` to the given buffer.
    ///
    /// Its default implementation just calls `.to_html()`, but you may
    /// override it with something more efficient.
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        self.to_html().push_html_to(builder)
    }
}

impl ToHtml for str {
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        // XSS-Safety: Special characters will be escaped by `escape_to_string`.
        escape::escape_to_string(self, builder.as_mut_string_unchecked());
    }
}

impl ToHtml for String {
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        str::push_html_to(self, builder);
    }
}

impl<'a> ToHtml for Cow<'a, str> {
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        str::push_html_to(self, builder);
    }
}

impl<'a> ToHtml for Arguments<'a> {
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        let _ = builder.write_fmt(*self);
    }
}

impl<'a, T: ToHtml + ?Sized> ToHtml for &'a T {
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        T::push_html_to(self, builder);
    }
}

impl<'a, T: ToHtml + ?Sized> ToHtml for &'a mut T {
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        T::push_html_to(self, builder);
    }
}

impl<T: ToHtml + ?Sized> ToHtml for Box<T> {
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        T::push_html_to(self, builder);
    }
}

macro_rules! impl_to_html_with_display {
    ($($ty:ty)*) => {
        $(
            impl ToHtml for $ty {
                fn push_html_to(&self, builder: &mut HtmlBuilder) {
                    let _ = write!(builder, "{self}");
                }
            }
        )*
    };
}

impl_to_html_with_display! {
    char f32 f64
}

macro_rules! impl_to_html_with_itoa {
    ($($ty:ty)*) => {
        $(
            impl ToHtml for $ty {
                fn push_html_to(&self, builder: &mut HtmlBuilder) {
                    // XSS-Safety: The characters '0' through '9', and '-', are HTML safe.
                    let _ = itoa::fmt(builder.as_mut_string_unchecked(), *self);
                }
            }
        )*
    };
}

impl_to_html_with_itoa! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
}

/// A fragment of HTML.
///
/// This is the type that's returned by the [`html!`] macro.
///
/// # Security
///
/// All instances of `Html` must be:
///
/// 1. **Trusted.** Any embedded scripts (in a `<script>` element or
///    otherwise) must come from a developer or admin, not an arbitrary
///    user.
///
/// 2. **Composable.** Appending two valid `Html` values must result in
///     another valid `Html`. This excludes, for example, unclosed tags.
///
/// Not following these rules can lead to [cross-site scripting attacks
/// (XSS)][xss].
///
/// [xss]: https://www.cloudflare.com/en-au/learning/security/threats/cross-site-scripting/
///
/// In general, the `html!` macro will enforce these rules automatically
/// (but see [caveats]). If you use the `_unchecked` methods, however,
/// you'll need to enforce these rules yourself.
///
/// [caveats]: https://maud.lambda.xyz/text-escaping.html#inline-script-and-style
///
/// # Which constructor should I use?
///
/// Most of the time, you should use the `html!` macro. Otherwise:
///
/// - **I need to add `<!DOCTYPE html>`.**
///     - Use the [`DOCTYPE`] constant.
///
/// - **I have an untrusted input (e.g. a Markdown comment on a blog).**
///     - Use [`Html::sanitize`].
///
/// - **I have a pre-defined snippet that I need to include in the page
///   (e.g. Google Analytics).**
///     - Use [`Html::from_const_unchecked`].
///
/// - **I have performance-sensitive rendering code that needs direct
///   access to the buffer.**
///     - Use [`HtmlBuilder::as_mut_string_unchecked`], and ask a
///       security expert for review.
///
/// - **I have special requirements and the other options don't work for
///   me.**
///     - Use [`Html::from_unchecked`], and ask a security expert for
///       review.
#[derive(Clone, Debug, Default)]
pub struct Html {
    inner: Cow<'static, str>,
}

impl Html {
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
    /// It is strongly recommended to include a `// XSS-Safety:` comment
    /// that explains why this call is safe.
    ///
    /// If your organization has a security team, consider asking them
    /// for review.
    pub fn from_unchecked(html_string: impl Into<Cow<'static, str>>) -> Self {
        Self {
            inner: html_string.into(),
        }
    }

    /// Creates an HTML fragment from a constant string.
    ///
    /// This is similar to [`Html::from_unchecked`], but can be called
    /// in a `const` context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use maud::Html;
    ///
    /// const ANALYTICS_SCRIPT: Html = Html::from_const_unchecked(
    ///     "<script>trackThisPageView();</script>",
    /// );
    /// ```
    ///
    /// # Security
    ///
    /// As long as the string is a compile-time constant, it is
    /// guaranteed to be as *trusted* as its surrounding code.
    ///
    /// However, this doesn't guarantee that it's *composable*:
    ///
    /// ```rust
    /// use maud::Html;
    ///
    /// // BROKEN - DO NOT USE!
    /// const UNCLOSED_SCRIPT: Html = Html::from_const_unchecked("<script>");
    /// const UNCLOSED_HREF: Html = Html::from_const_unchecked("<a href='");
    /// ```
    ///
    /// This is why the method has an `_unchecked` prefix -- you must
    /// verify that the HTML is valid yourself.
    pub const fn from_const_unchecked(html_string: &'static str) -> Self {
        Html {
            inner: Cow::Borrowed(html_string),
        }
    }

    /// Exposes the underlying buffer as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Converts the inner value to a `String`.
    pub fn into_string(self) -> String {
        self.inner.into_owned()
    }
}

impl ToHtml for Html {
    fn push_html_to(&self, builder: &mut HtmlBuilder) {
        // XSS-Safety: `self` is already guaranteed to be trusted HTML.
        builder.as_mut_string_unchecked().push_str(self.as_str());
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
pub const DOCTYPE: Html = Html::from_const_unchecked("<!DOCTYPE html>");

/// A partially created fragment of HTML.
///
/// Unlike [`Html`], an `HtmlBuilder` might have unclosed elements or
/// attributes.
///
/// This type cannot be constructed by hand. The [`html!`] macro creates
/// one internally, and passes it to [`ToHtml::push_html_to`].
#[derive(Clone, Debug)]
pub struct HtmlBuilder {
    inner: Cow<'static, str>,
}

impl HtmlBuilder {
    /// For internal use only.
    #[doc(hidden)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: Cow::Owned(String::new()),
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
        value.push_html_to(self);
    }

    /// Exposes the underlying buffer as a `&mut String`.
    ///
    /// This is useful for performance-sensitive use cases that need
    /// direct access to the buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use maud::{HtmlBuilder, ToHtml};
    /// # mod base64 { pub fn encode(_: &mut String, _: &[u8]) {} }
    ///
    /// struct Base64<'a>(&'a [u8]);
    ///
    /// impl<'a> ToHtml for Base64<'a> {
    ///     fn push_html_to(&self, builder: &mut HtmlBuilder) {
    ///         // XSS-Safety: The characters [A-Za-z0-9+/=] are all HTML-safe.
    ///         base64::encode(builder.as_mut_string_unchecked(), self.0);
    ///     }
    /// }
    /// ```
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

    /// For internal use only.
    #[doc(hidden)]
    pub fn finalize(self) -> Html {
        // XSS-Safety: This is called from the `html!` macro, which enforces safety itself.
        Html::from_unchecked(self.inner)
    }
}

impl Write for HtmlBuilder {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.push(text);
        Ok(())
    }
}

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
