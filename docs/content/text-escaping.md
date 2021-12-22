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
assert_eq!(markup.into_string(), "&lt;p&gt;Pickle, barrel, kumquat.&lt;/p&gt;");
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

[xss]: https://www.cloudflare.com/en-au/learning/security/threats/cross-site-scripting/

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

## Inline `<script>` and `<style>`

In HTML,
`<script>` and `<style>` elements
have [special syntax rules].

[special syntax rules]: https://html.spec.whatwg.org/multipage/scripting.html#restrictions-for-contents-of-script-elements

Maud does not yet implement these special rules,
so it's not recommended to write `script { ... }` or `style { ... }` directly.

Instead, either:

- Put the CSS or JavaScript in a separate file,
  and link to it:

  ```rust
  # let _ = maud::
  html! {
      script src="my-external-script.js" {}
  }
  # ;
  ```

- Wrap the whole thing in [`Html::from_const_unchecked`][from_const_unchecked],
  to bypass Maud's escaping:

  ```rust
  use maud::Html;
  # let _ = maud::
  html! {
      (Html::from_const_unchecked("<script>doCoolStuff();</script>"))
  }
  # ;
  ```

[from_const_unchecked]: https://docs.rs/maud/*/maud/struct.Html.html#method.from_const_unchecked

When Maud implements [context-aware escaping],
these workarounds will no longer be needed.

[context-aware escaping]: https://github.com/lambda-fairy/maud/issues/181

## Custom escaping

If your use case isn't covered by these examples,
check out the [advanced API].

[advanced API]: https://docs.rs/maud/*/maud/struct.Html.html
