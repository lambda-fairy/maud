# Getting started

## Add Maud to your project

Once Rust is set up, create a new project with Cargo:

```sh
cargo new --bin pony-greeter
cd pony-greeter
```

Add `maud` to your `Cargo.toml`:

```toml
[dependencies]
maud = "*"
```

Then save the following to `src/main.rs`:

```rust
use maud::html;

fn main() {
    let name = "Lyra";
    let markup = html! {
        p { "Hi, " (name) "!" }
    };
    println!("{}", markup.into_string());
}
```

`html!` takes a single argument: a template using Maud's custom syntax.
This call expands to an expression of type [`Markup`][Markup], which can then be converted to a `String` using `.into_string()`.

[Markup]: https://docs.rs/maud/*/maud/type.Markup.html

Run this program with `cargo run`, and you should get the following:

```html
<p>Hi, Lyra!</p>
```

Congrats – you've written your first Maud program!

## Which version of Rust?

While Maud works well on both stable and [nightly] versions of Rust, the error messages are slightly better on nightly.
For this reason, it is recommended to develop using nightly Rust, but test and deploy using stable.

[nightly]: https://doc.rust-lang.org/book/appendix-07-nightly-rust.html
