# Dynamic content

Use `(foo)` syntax to splice in the value of `foo` at runtime. Any HTML special characters are escaped by default.

```rust
let best_pony = "Pinkie Pie";
let numbers = [1, 2, 3, 4];
html! {
    p { "Hi, " (best_pony) "!" }
    p {
        "I have " (numbers.len()) " numbers, "
        "and the first one is " (numbers[0])
    }
}
```

Arbitrary Rust code can be included in a splice by using a [block](https://doc.rust-lang.org/reference.html#block-expressions). This can be helpful for complex expressions that would be difficult to read otherwise.

```rust
html! {
    p {
        ({
            let f: Foo = something_convertible_to_foo()?;
            f.time().format("%H%Mh")
        })
    }
}
```

## Splices in attributes

Splices work in attributes as well.

```rust
let secret_message = "Surprise!";
html! {
    p title=(secret_message) {
        "Nothing to see here, move along."
    }
}
```

To concatenate multiple values within an attribute, wrap the whole thing in braces. This syntax is useful for building URLs.

```rust
const GITHUB: &'static str = "https://github.com";
html! {
    a href={ (GITHUB) "/lfairy/maud" } {
        "Fork me on GitHub"
    }
}
```

## What can be spliced?

You can splice any value that implements [`std::fmt::Display`][Display]. Most primitive types (such as `str` and `i32`) implement this trait, so they should work out of the box.

To change this behavior for some type, you can implement the [`Render`][Render] trait by hand. The [`PreEscaped`][PreEscaped] wrapper type, which outputs its argument without escaping, works this way. See the [traits](./traits.md) section for details.

```rust
use maud::PreEscaped;
let post = "<p>Pre-escaped</p>";
html! {
    h1 { "My super duper blog post" }
    (PreEscaped(post))
}
```

[Display]: http://doc.rust-lang.org/std/fmt/trait.Display.html
[Render]: https://docs.rs/maud/*/maud/trait.Render.html
[PreEscaped]: https://docs.rs/maud/*/maud/struct.PreEscaped.html
