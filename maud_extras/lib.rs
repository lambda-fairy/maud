#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;

use maud::{Markup, Render};

/// Links to an external stylesheet.
///
/// # Example
///
/// ```rust
/// # #![feature(plugin)]
/// # #![plugin(maud_macros)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud_extras::*;
/// # fn main() {
/// let markup = html! { (Css("styles.css")) };
/// assert_eq!(markup.into_string(),
///            r#"<link rel="stylesheet" href="styles.css">"#);
/// # }
/// ```
pub struct Css<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for Css<T> {
    fn render(&self) -> Markup {
        html! {
            link rel="stylesheet" href=(self.0.as_ref()) /
        }
    }
}

/// Links to an external javascript.
///
/// # Example
///
/// ```rust
/// # #![feature(plugin)]
/// # #![plugin(maud_macros)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud_extras::*;
/// # fn main() {
/// let markup = html! { (Js("app.js")) };
/// assert_eq!(markup.into_string(),
///            r#"<script src="app.js"></script>"#);
/// # }
/// ```
pub struct Js<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for Js<T> {
    fn render(&self) -> Markup {
        html! {
            script src=(self.0.as_ref()) {}
        }
    }
}
