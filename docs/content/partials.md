# Partials

Maud does not have a built-in concept of partials or sub-templates. Instead, you can compose your markup with any function that returns `Markup`.

The following example uses a `header` and `footer` function that are used in the `page` function to return a final result.

```rust
extern crate maud;

use self::maud::{DOCTYPE, html, Markup};

/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Markup {
    html! {
        (DOCTYPE)
        html {
            meta charset="utf-8";
            title { (page_title) }
        }
    }
}

/// A static footer.
fn footer() -> Markup {
    html! {
        footer {
            span {
                a href="rss.atom" { "RSS Feed" }
            }
        }
    }
}

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(title: &str, greeting_box: Markup) -> Markup {
    html! {
        // Add the header markup to the page
        (header(title))
        body {
            h1 { "Hello World" }
            (greeting_box)
        }
        // Add the footer markup to the page
        (footer())
    }
}
```

Using the `page` function will return the markup for the whole page and looks like this:

```rust
fn main() {
    page("Hello!", html! {
        div { "Greetings, Maud." }
    });
}
```
