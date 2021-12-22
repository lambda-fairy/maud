# Partials

Maud does not have a built-in concept of partials or sub-templates.
Instead,
you can compose your markup with any function that returns `Html`.

The following example defines a `header` and `footer` function.
These functions are combined to form the final `page`.

```rust
use maud::{DOCTYPE, html, Html};

/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Html {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title { (page_title) }
    }
}

/// A static footer.
fn footer() -> Html {
    html! {
        footer {
            a href="rss.atom" { "RSS Feed" }
        }
    }
}

/// The final page, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Html`, not `&str`.
pub fn page(title: &str, greeting_box: Html) -> Html {
    html! {
        (header(title))
        h1 { (title) }
        (greeting_box)
        (footer())
    }
}
```

Using the `page` function will return the HTML for the whole page.
Here's an example:

```rust
# use maud::{html, Html};
# fn page(title: &str, greeting_box: Html) -> Html { greeting_box }
page("Hello!", html! {
    div { "Greetings, Maud." }
});
```
