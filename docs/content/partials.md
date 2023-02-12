# Partials

Maud does not have a built-in concept of partials or sub-templates.
Instead, you can compose your markup with any function that returns `Markup`.

The following example defines a `header` and `footer` function.
These functions are combined to form the final `page`.

```rust
use maud::{DOCTYPE, html, Markup};

/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title { (page_title) }
    }
}

/// A static footer.
fn footer() -> Markup {
    html! {
        footer {
            a href="rss.atom" { "RSS Feed" }
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
        h1 { (title) }
        (greeting_box)
        (footer())
    }
}
```

Using the `page` function will return the markup for the whole page.
Here's an example:

```rust
# use maud::{html, Markup};
# fn page(title: &str, greeting_box: Markup) -> Markup { greeting_box }
page("Hello!", html! {
    div { "Greetings, Maud." }
});
```
