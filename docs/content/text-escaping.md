# Text and escaping

## Text

Literal strings use the same syntax as Rust.
Wrap them in double quotes,
and use a backslash for escapes.

```rust
# let _ = maud::
html! {
    "Oatmeal, are you crazy?"
}
# ;
```

## Raw strings

If the string is long,
or contains many special characters,
then it may be worth using [raw strings] instead:

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

## Escaping

By default,
HTML special characters are escaped automatically.

```rust
# use maud::html;
let markup = html! {
    "<p>Pickle, barrel, kumquat.</p>"
};
assert_eq!(markup.into_string(), "&lt;p&gt;Pickel, barrel, kumquat.&lt;/p&gt;");
```

This escaping also applies within a [splice](splices-toggles.md),
which prevents [cross-site scripting][xss] attacks:

```rust
# use maud::html;
let unsafe_input = "<script>alert('Bwahahaha!')</script>";
let markup = html! {
    (unsafe_input)
};
assert_eq!(markup.into_string(), "&lt;script&gt;alert('Bwahahaha!')&lt;/script&gt;");
```

[xss]: https://owasp.org/www-community/attacks/xss/

## The `DOCTYPE` constant

If you want to add a `<!DOCTYPE html>` declaration to your page,
you may use the `maud::DOCTYPE` constant
instead of writing it out by hand:

```rust
use maud::DOCTYPE;
# let _ = maud::
html! {
    (DOCTYPE)  // <!DOCTYPE html>
}
# ;
```
