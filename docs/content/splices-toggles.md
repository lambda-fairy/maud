# Splices and toggles

## Splices: `(foo)`

Use `(foo)` syntax to insert the value of `foo` at runtime.
Any HTML special characters are escaped by default.

```rust
let best_pony = "Pinkie Pie";
let numbers = [1, 2, 3, 4];
# let _ = maud::
html! {
    p { "Hi, " (best_pony) "!" }
    p {
        "I have " (numbers.len()) " numbers, "
        "and the first one is " (numbers[0])
    }
}
# ;
```

Arbitrary Rust code can be included in a splice by using a [block].
This can be helpful for complex expressions that would be difficult to read otherwise.

```rust
# struct Foo;
# impl Foo { fn time(self) -> Bar { Bar } }
# struct Bar;
# impl Bar { fn format(self, _: &str) -> &str { "" } }
# fn something_convertible_to_foo() -> Option<Foo> { Some(Foo) }
# fn test() -> Option<()> {
# let _ = maud::
html! {
    p {
        ({
            let f: Foo = something_convertible_to_foo()?;
            f.time().format("%H%Mh")
        })
    }
}
# ;
# Some(())
# }
```

[block]: https://doc.rust-lang.org/reference.html#block-expressions

### Splices in attributes

Splices work in attributes as well.

```rust
let secret_message = "Surprise!";
# let _ = maud::
html! {
    p title=(secret_message) {
        "Nothing to see here, move along."
    }
}
# ;
```

To concatenate multiple values within an attribute, wrap the whole thing in braces.
This syntax is useful for building URLs.

```rust
const GITHUB: &'static str = "https://github.com";
# let _ = maud::
html! {
    a href={ (GITHUB) "/lambda-fairy/maud" } {
        "Fork me on GitHub"
    }
}
# ;
```

### Splices in classes and IDs

Splices can also be used in classes and IDs.

```rust
let name = "rarity";
let severity = "critical";
# let _ = maud::
html! {
    aside #(name) {
        p.{ "color-" (severity) } { "This is the worst! Possible! Thing!" }
    }
}
# ;
```

### What can be spliced?

You can splice any value that implements [`Render`][Render].
Most primitive types (such as `str` and `i32`) implement this trait, so they should work out of the box.

To get this behavior for a custom type, you can implement the [`Render`][Render] trait by hand.
The [`PreEscaped`][PreEscaped] wrapper type, which outputs its argument without escaping, works this way.
See the [traits](render-trait.md) section for details.

```rust
use maud::PreEscaped;
let post = "<p>Pre-escaped</p>";
# let _ = maud::
html! {
    h1 { "My super duper blog post" }
    (PreEscaped(post))
}
# ;
```

[Render]: https://docs.rs/maud/*/maud/trait.Render.html
[PreEscaped]: https://docs.rs/maud/*/maud/struct.PreEscaped.html

## Toggles: `[foo]`

Use `[foo]` syntax to show or hide something based on a boolean expression `foo`.

This works on empty attributes:

```rust
let allow_editing = true;
# let _ = maud::
html! {
    p contenteditable[allow_editing] {
        "Edit me, I "
        em { "dare" }
        " you."
    }
}
# ;
```

And classes:

```rust
let cuteness = 95;
# let _ = maud::
html! {
    p.cute[cuteness > 50] { "Squee!" }
}
# ;
```
