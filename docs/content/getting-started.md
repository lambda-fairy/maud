# Getting started

## Install nightly Rust

Maud requires the nightly version of Rust.
If you're using `rustup`,
see the [documentation][rustup]
for how to install this version.

[rustup]: https://github.com/rust-lang/rustup.rs/blob/master/README.md#working-with-nightly-rust

## Add Maud to your project

Once Rust is set up,
create a new project with Cargo:

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
extern crate maud;
use maud::html;

fn main() {
    let name = "Lyra";
    let markup = html! {
        p { "Hi, " (name) "!" }
    };
    println!("{}", markup.into_string());
}
```

`html!` takes a single argument: a template using Maud's custom syntax. This call expands to an expression of type [`Markup`][Markup], which can then be converted to a `String` using `.into_string()`.

[Markup]: https://docs.rs/maud/*/maud/type.Markup.html

Run this program with `cargo run`, and you should get the following:

```
<p>Hi, Lyra!</p>
```

Congrats – you've written your first Maud program!
