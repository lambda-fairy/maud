//! A macro for writing HTML templates.
//!
//! This crate, along with its sister `maud_macros`, lets you generate
//! HTML markup from within Rust. It exposes a single macro, `html!`,
//! which compiles your markup to specialized Rust code.
//!
//! # Dependencies
//!
//! To get started, add `maud` and `maud_macros` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! # ...
//! maud = "*"
//! maud_macros = "*"
//! ```
//!
//! # Example
//!
//! ```rust
//! #![feature(plugin)]
//! #![allow(unstable)]
//!
//! extern crate maud;
//! #[plugin] #[no_link] extern crate maud_macros;
//!
//! fn main() {
//!     let name = "Lyra";
//!     let markup = html! {
//!         p { "Hi, " $name "!" }
//!     };
//!     assert_eq!(markup.render(), "<p>Hi, Lyra!</p>");
//! }
//! ```
//!
//! # Syntax
//!
//! Maud uses its own syntax, loosely inspired by HTML and Rust.
//!
//! ## Literals
//!
//! ```
//! html! {
//!     "Oatmeal, are you crazy?\n"
//!     "<script>alert(\"XSS\")</script>\n"
//!     $$"<script>alert(\"XSS\")</script>\n"
//! }
//! ```
//!
//! ```html
//! Oatmeal, are you crazy?
//! &lt;script&gt;alert(&quot;XSS&quot;)&lt;/script&gt;
//! <script>alert("XSS")</script>
//! ```
//!
//! Literal strings use the same syntax as Rust.
//!
//! By default, HTML special characters are escaped automatically. Add a
//! `$$` prefix to disable this escaping.
//!
//! ## Elements
//!
//! ```
//! html! {
//!     h1 "Pinkie's Brew"
//!     p {
//!         "Watch as I work my gypsy magic"
//!         br /
//!         "Eye of a newt and cinnamon"
//!     }
//!     p small em "squee"
//! }
//! ```
//!
//! ```html
//! <h1>Pinkie's Brew</h1>
//! <p>
//!   Watch as I work my gypsy magic
//!   <br>
//!   Eye of a newt and cinnamon
//! </p>
//! <p><small><em>squee</em></small></p>
//! ```
//!
//! Write an element using curly braces (`p {}`).
//!
//! Terminate a void element using a slash (`br /`).
//!
//! If the element has only a single child, you can omit the brackets
//! (`h1 "Pinkie's Brew"`). This shorthand works with nested elements
//! too.
//!
//! ## Attributes
//!
//! ```
//! html! {
//!     form method="POST" {
//!         label for="waffles" "Do you like waffles?"
//!         input name="waffles" type="checkbox" checked=! /
//!     }
//! }
//! ```
//!
//! ```html
//! <form method="POST">
//!   <label for="waffles">Do you like waffles?</label>
//!   <input name="waffles" type="checkbox" checked>
//! </form>
//! ```
//!
//! Add attributes using the syntax `attr="value"`. Attributes must be
//! quoted: they are parsed as string literals.
//!
//! To declare an empty attribute, use `!` for the value: `checked=!`.
//!
//! ## Splices
//!
//! ```
//! let best_pony = "Pinkie Pie";
//! let numbers = [1, 2, 3, 4];
//! let secret_message = "Surprise!";
//! let pre_escaped = "<p>Pre-escaped</p>";
//! html! {
//!     h1 { $best_pony " says:" }
//!     p {
//!         "I have " $numbers.len() " numbers, "
//!         "and the first one is " $numbers[0]
//!     }
//!     p title=$secret_message {
//!         "1 + 1 = " $(1 + 1)
//!     }
//!     $$pre_escaped
//! }
//! ```
//!
//! ```html
//! <h1>Pinkie Pie says:</h1>
//! <p>I have 4 numbers, and the first one is 1</p>
//! <p title="Surprise!">1 + 1 = 2</p>
//! <p>Pre-escaped</p>
//! ```
//!
//! Use `$(expr)` syntax to splice a Rust expression into the output.
//! You can omit the brackets if it's just a variable (`$var`), indexing
//! operation (`$var[0]`), method call (`$var.method()`), or property
//! lookup (`$var.property`).
//!
//! As with literals, expression values are escaped by default. Use a
//! `$$` prefix to disable this behavior.

#![allow(unstable)]

use std::fmt;
use std::io::{IoError, IoErrorKind, IoResult};

/// Escape an HTML value.
pub fn escape(s: &str) -> String {
    use std::fmt::Writer;
    let mut buf = String::new();
    rt::Escaper { inner: &mut buf }.write_str(s).unwrap();
    buf
}

/// A block of HTML markup, as returned by the `html!` macro.
///
/// Use `.render()` to convert it to a `String`, or `.render_to()` to
/// write it directly to a handle.
pub struct Markup<'a> {
    callback: &'a (Fn(&mut fmt::Writer) -> fmt::Result + 'a),
}

impl<'a> Markup<'a> {
    /// Render the markup to a `String`.
    pub fn render(&self) -> String {
        let mut buf = String::new();
        self.render_fmt(&mut buf).unwrap();
        buf
    }

    /// Render the markup to a `std::io::Writer`.
    ///
    /// If you're writing to, say, a file or a socket, this may be more
    /// efficient than calling `.render()`.
    pub fn render_to(&self, w: &mut Writer) -> IoResult<()> {
        struct WriterWrapper<'a, 'b: 'a> {
            inner: &'a mut (Writer + 'b),
        }
        impl<'a, 'b> fmt::Writer for WriterWrapper<'a, 'b> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.inner.write_str(s).map_err(|_| fmt::Error)
            }
        }
        self.render_fmt(&mut WriterWrapper { inner: w })
            .map_err(|_| IoError {
                kind: IoErrorKind::OtherIoError,
                desc: "formatting error",
                detail: None,
            })
    }

    /// Render the markup to a `std::fmt::Writer`.
    pub fn render_fmt(&self, w: &mut fmt::Writer) -> fmt::Result {
        (self.callback)(w)
    }
}

/// Internal functions used by the `maud_macros` package. You should
/// never need to call these directly.
#[doc(hidden)]
pub mod rt {
    use std::fmt;
    use super::Markup;

    #[inline]
    pub fn make_markup<'a>(f: &'a (Fn(&mut fmt::Writer) -> fmt::Result + 'a)) -> Markup<'a> {
        Markup { callback: f }
    }

    /// rustc is a butt and doesn't let us quote macro invocations
    /// directly. So we factor the `write!` call into a separate
    /// function and use that instead.
    ///
    /// See <https://github.com/rust-lang/rust/issues/16617>
    #[inline]
    pub fn write_fmt<T: fmt::Display>(w: &mut fmt::Writer, value: T) -> fmt::Result {
        write!(w, "{}", value)
    }

    pub struct Escaper<'a, 'b: 'a> {
        pub inner: &'a mut (fmt::Writer + 'b),
    }

    impl<'a, 'b> fmt::Writer for Escaper<'a, 'b> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for c in s.chars() {
                try!(match c {
                    '&' => self.inner.write_str("&amp;"),
                    '<' => self.inner.write_str("&lt;"),
                    '>' => self.inner.write_str("&gt;"),
                    '"' => self.inner.write_str("&quot;"),
                    '\'' => self.inner.write_str("&#39;"),
                    _ => write!(self.inner, "{}", c),
                });
            }
            Ok(())
        }
    }
}
