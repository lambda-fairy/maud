# Elements and attributes

## Elements: `p`

Write an element using curly braces: `p { ... }`.

Terminate a void element using a semicolon: `br;`. Note that the result will be rendered with HTML syntax – `<br>` not `<br />`.

```rust
html! {
    h1 { "Poem" }
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
```

Maud also supports ending a void element with a slash: `br /`. This syntax is [deprecated][#96] and should not be used in new code.

[#96]: https://github.com/lambda-fairy/maud/pull/96

Before version 0.18, Maud allowed the curly braces to be omitted. This syntax was [removed][#137] and now causes an error instead.

[#137]: https://github.com/lambda-fairy/maud/pull/137

## Custom elements

Maud also supports [custom elements].

```rust
html! {
    blog-post {
        title { "My blog" }
    }
}
```

[custom elements]: https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_custom_elements

## Non-empty attributes: `title="yay"`

Add attributes using the syntax: `attr="value"`. You can attach any number of attributes to an element. The values must be quoted: they are parsed as string literals.

```rust
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
```

## Empty attributes: `checked`

Declare an empty attribute by omitting the value.

```rust
html! {
    form {
        input type="checkbox" name="cupcakes" checked;
        " "
        label for="cupcakes" { "Do you like cupcakes?" }
    }
}
```

Before version 0.22.2, Maud required a `?` suffix on empty attributes: `checked?`. This is no longer necessary ([#238]), but still supported for backward compatibility.

[#238]: https://github.com/lambda-fairy/maud/pull/238

## Classes and IDs: `.foo` `#bar`

Add classes and IDs to an element using `.foo` and `#bar` syntax. You can chain multiple classes and IDs together, and mix and match them with other attributes:

```rust
html! {
    input#cannon.big.scary.bright-red type="button" value="Launch Party Cannon";
}
```

## Implicit `div` elements

If the element name is omitted, but there is a class or ID, then it is assumed to be a `div`.

```rust
html! {
    #main {
        "Main content!"
    }
}
```
