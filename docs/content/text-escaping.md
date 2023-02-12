# Text and escaping

## Text

Literal strings use the same syntax as Rust.
Wrap them in double quotes, and use a backslash for escapes.

```rust
# let _ = maud::
html! {
    "Oatmeal, are you crazy?"
}
# ;
```

## Raw strings

If the string is long, or contains many special characters, then it may be worth using [raw strings] instead:

```rust
# let _ = maud::
html! {
    pre {
        r#"
            Rocks, these are my rocks.
            Sediments make me sedimental.
            Smooth and round,
            Asleep in the ground.
            Shades of brown
            And gray.
        "#
    }
}
# ;
```

[raw strings]: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals

## Escaping and `PreEscaped`

By default, HTML special characters are escaped automatically.
Wrap the string in `(PreEscaped())` to disable this escaping.
(See the section on [splices](splices-toggles.md) to learn more about how this works.)

```rust
use maud::PreEscaped;
# let _ = maud::
html! {
    "<script>alert(\"XSS\")</script>"                // &lt;script&gt;...
    (PreEscaped("<script>alert(\"XSS\")</script>"))  // <script>...
}
# ;
```

## The `DOCTYPE` constant

If you want to add a `<!DOCTYPE html>` declaration to your page, you may use the `maud::DOCTYPE` constant instead of writing it out by hand:

```rust
use maud::DOCTYPE;
# let _ = maud::
html! {
    (DOCTYPE)  // <!DOCTYPE html>
}
# ;
```
