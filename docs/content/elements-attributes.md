# Elements and attributes

## Elements with contents: `p {}`

Write an element using curly braces:

```rust
# let _ = maud::
html! {
    h1 { "Poem" }
    p {
        strong { "Rock," }
        " you are a rock."
    }
}
# ;
```

## Void elements: `br;`

Terminate a void element using a semicolon:

```rust
# let _ = maud::
html! {
    link rel="stylesheet" href="poetry.css";
    p {
        "Rock, you are a rock."
        br;
        "Gray, you are gray,"
        br;
        "Like a rock, which you are."
        br;
        "Rock."
    }
}
# ;
```

The result will be rendered with HTML syntax – `<br>` not `<br />`.

## Custom elements and `data` attributes

Maud also supports elements and attributes with hyphens in them.
This includes [custom elements], [data attributes], and [ARIA annotations].

```rust
# let _ = maud::
html! {
    article data-index="12345" {
        h1 { "My blog" }
        tag-cloud { "pinkie pie pony cute" }
    }
}
# ;
```

[custom elements]: https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_custom_elements
[data attributes]: https://css-tricks.com/a-complete-guide-to-data-attributes/
[ARIA annotations]: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Annotations

## Non-empty attributes: `title="yay"`

Add attributes using the syntax: `attr="value"`.
You can attach any number of attributes to an element.
The values must be quoted: they are parsed as string literals.

```rust
# let _ = maud::
html! {
    ul {
        li {
            a href="about:blank" { "Apple Bloom" }
        }
        li class="lower-middle" {
            "Sweetie Belle"
        }
        li dir="rtl" {
            "Scootaloo "
            small { "(also a chicken)" }
        }
    }
}
# ;
```

## Optional attributes: `title=[Some("value")]`

Add optional attributes to an element using `attr=[value]` syntax, with *square* brackets.
These are only rendered if the value is `Some<T>`, and entirely omitted if the value is `None`.

```rust
# let _ = maud::
html! {
    p title=[Some("Good password")] { "Correct horse" }

    @let value = Some(42);
    input value=[value];

    @let title: Option<&str> = None;
    p title=[title] { "Battery staple" }
}
# ;
```

## Empty attributes: `checked`

Declare an empty attribute by omitting the value.

```rust
# let _ = maud::
html! {
    form {
        input type="checkbox" name="cupcakes" checked;
        " "
        label for="cupcakes" { "Do you like cupcakes?" }
    }
}
# ;
```

Before version 0.22.2, Maud required a `?` suffix on empty attributes: `checked?`.
This is no longer necessary ([#238]), but still supported for backward compatibility.

[#238]: https://github.com/lambda-fairy/maud/pull/238

## Classes and IDs: `.foo` `#bar`

Add classes and IDs to an element using `.foo` and `#bar` syntax.
You can chain multiple classes and IDs together, and mix and match them with other attributes:

```rust
# let _ = maud::
html! {
    input #cannon .big.scary.bright-red type="button" value="Launch Party Cannon";
}
# ;
```

In Rust 2021, the `#` symbol must be preceded by a space, to avoid conflicts with [reserved syntax]:

[reserved syntax]: https://doc.rust-lang.org/edition-guide/rust-2021/reserving-syntax.html

```rust,edition2018
# let _ = maud::
html! {
    // Works on all Rust editions
    input #pinkie;

    // Works on Rust 2018 and older only
    input#pinkie;
}
# ;
```

The classes and IDs can be quoted.
This is useful for names with numbers or symbols which otherwise wouldn't parse:

```rust
# let _ = maud::
html! {
    div."col-sm-2" { "Bootstrap column!" }
}
# ;
```

## Implicit `div` elements

If the element name is omitted, but there is a class or ID, then it is assumed to be a `div`.

```rust
# let _ = maud::
html! {
    #main {
        "Main content!"
        .tip { "Storing food in a refrigerator can make it 20% cooler." }
    }
}
# ;
```
