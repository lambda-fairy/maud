#![feature(plugin)]
#![feature(proc_macro)]

#![plugin(maud_lints)]

extern crate maud;

use maud::{Markup, Render, html};

/// Links to an external stylesheet.
///
/// # Example
///
/// ```rust
/// # #![feature(proc_macro)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud::html;
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

/// Links to an external script.
///
/// # Example
///
/// ```rust
/// # #![feature(proc_macro)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud::html;
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

/// Generate a `<meta>` element.
///
/// # Example
///
/// ```rust
/// # #![feature(proc_macro)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud::html;
/// # use maud_extras::*;
/// # fn main() {
/// let markup = html! { (Meta("description", "test description")) };
/// assert_eq!(markup.into_string(),
///            r#"<meta name="description" content="test description">"#);
/// # }
/// ```
pub struct Meta<T: AsRef<str>, U: AsRef<str>>(pub T, pub U);

impl<T: AsRef<str>, U: AsRef<str>> Render for Meta<T, U> {
    fn render(&self) -> Markup {
        html! {
            meta name=(self.0.as_ref()) content=(self.1.as_ref()) /
        }
    }
}

/// Generate a `<title>` element.
///
/// # Example
///
/// ```rust
/// # #![feature(proc_macro)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud::html;
/// # use maud_extras::*;
/// # fn main() {
/// let markup = html! { (Title("Maud")) };
/// assert_eq!(markup.into_string(),
///            r#"<title>Maud</title>"#);
/// # }
/// ```
pub struct Title<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for Title<T> {
    fn render(&self) -> Markup {
        html! {
            title (self.0.as_ref())
        }
    }
}

/// Generate a `<meta charset="...">` element.
///
/// # Example
///
/// ```rust
/// # #![feature(proc_macro)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud::html;
/// # use maud_extras::*;
/// # fn main() {
/// let markup = html! { (Charset("utf-8")) };
/// assert_eq!(markup.into_string(),
///            r#"<meta charset="utf-8">"#);
/// # }
/// ```
pub struct Charset<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for Charset<T> {
    fn render(&self) -> Markup {
        html! {
            meta charset=(self.0.as_ref()) /
        }
    }
}

/// Generate a `<meta property>` element.
///
/// # Example
///
/// ```rust
/// # #![feature(proc_macro)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud::html;
/// # use maud_extras::*;
/// # fn main() {
/// let markup = html! { (MetaProperty("og:description", "test description")) };
/// assert_eq!(markup.into_string(),
///            r#"<meta property="og:description" content="test description">"#);
/// # }
/// ```
pub struct MetaProperty<T: AsRef<str>, U: AsRef<str>>(pub T, pub U);

impl<T: AsRef<str>, U: AsRef<str>> Render for MetaProperty<T, U> {
    fn render(&self) -> Markup {
        html! {
            meta property=(self.0.as_ref()) content=(self.1.as_ref()) /
        }
    }
}

/// Generate a `<meta robots>` element.
///
/// # Example
///
/// ```rust
/// # #![feature(proc_macro)]
/// # extern crate maud;
/// # extern crate maud_extras;
/// # use maud::html;
/// # use maud_extras::*;
/// # fn main() {
/// let markup = html! { (MetaRobots { index: true, follow: false }) };
/// assert_eq!(markup.into_string(),
///            r#"<meta name="robots" content="index,nofollow">"#);
/// # }
/// ```
pub struct MetaRobots {
    pub index: bool,
    pub follow: bool,
}

impl Render for MetaRobots {
    fn render(&self) -> Markup {
        let index = if self.index { "index" } else { "noindex" };
        let follow = if self.follow { "follow" } else { "nofollow" };
        html! {
            meta name="robots" content={ (index) "," (follow) } /
        }
    }
}
