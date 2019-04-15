# Basic syntax

The next few sections will outline the syntax used by Maud templates.

## Literals `""`

Literal strings use the same syntax as Rust. Wrap them in double quotes, and use a backslash for escapes.

```rust
html! {
    "Oatmeal, are you crazy?"
}
```

### Escaping and `PreEscaped`

By default, HTML special characters are escaped automatically. Wrap the string in `(PreEscaped())` to disable this escaping. (See the section on [dynamic content] to learn more about how this works.)

```rust
use maud::PreEscaped;
html! {
    "<script>alert(\"XSS\")</script>"                // &lt;script&gt;...
    (PreEscaped("<script>alert(\"XSS\")</script>"))  // <script>...
}
```

[dynamic content]: dynamic-content.md

If the string is long, or contains many special characters, then it may be worth using [raw strings] instead:

```rust
use maud::PreEscaped;
html! {
    (PreEscaped(r#"
        <script>
            alert("Look ma, no backslashes!");
        </script>
    "#))
}
```

[raw strings]: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals

If you want to add a `<!DOCTYPE html>` declaration to your page, you may use the `maud::DOCTYPE` constant instead of writing it out by hand:

```rust
use maud::DOCTYPE;
html! {
    (DOCTYPE)  // <!DOCTYPE html>
}
```

## Elements `p`

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

[#96]: https://github.com/lfairy/maud/pull/96

Before version 0.18, Maud allowed the curly braces to be omitted. This syntax was [removed][#137] and now causes an error instead.

[#137]: https://github.com/lfairy/maud/pull/137

## Non-empty attributes `id="yay"`

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

Add attributes using the syntax: `attr="value"`. You can attach any number of attributes to an element. The values must be quoted: they are parsed as string literals.

## Empty attributes `checked?` `disabled?[foo]`

Declare an empty attribute using a `?` suffix: `checked?`.

```rust
html! {
    form {
        input type="checkbox" name="cupcakes" checked?;
        " "
        label for="cupcakes" { "Do you like cupcakes?" }
    }
}
```

To toggle an attribute based on a boolean flag, use a `?[]` suffix instead: `checked?[foo]`. This will check the value of `foo` at runtime, inserting the attribute only if `foo` equals `true`.

```rust
let allow_editing = true;
html! {
    p contenteditable?[allow_editing] {
        "Edit me, I "
        em { "dare" }
        " you."
    }
}
```

## Classes and IDs `.foo` `#bar`

Add classes and IDs to an element using `.foo` and `#bar` syntax. The tag will default to `div` if an element begins with a class or ID. You can chain multiple classes and IDs together, and mix and match them with other attributes:

```rust
html! {
    .container#main {
        input.big.scary.bright-red type="button" value="Launch Party Cannon";
    }
}
```

To toggle a class based on a boolean flag, use a `[]` suffix: `.foo[is_foo]`. This will check the value of `is_foo` at runtime, inserting that class value `foo` in the class attribute only if `is_foo` is `true`.

```rust
let cuteness = 95;
html! {
    p.cute[cuteness > 50] { "Squee!" }
}
```
