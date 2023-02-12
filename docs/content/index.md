<!-- Comment that prevents the title from getting picked up -->

# A macro for writing HTML

```rust
# let _ = maud::
html! {
    h1 { "Hello, world!" }
    p.intro {
        "This is an example of the "
        a href="https://github.com/lambda-fairy/maud" { "Maud" }
        " template language."
    }
}
# ;
```

Maud is an HTML [template engine] for Rust.
It's implemented as a macro, `html!`, which compiles your markup to specialized Rust code.
This unique approach makes Maud templates fast, type-safe, and easy to deploy.

[template engine]: https://www.simple-is-better.org/template/

## Tight integration with Rust

Since Maud is a Rust macro, it can borrow most of its features from the host language.
Pattern matching and `for` loops work as they do in Rust.
There is no need to derive JSON conversions, as your templates can work with Rust values directly.

## Type safety

Your templates are checked by the compiler, just like the code around them.
Any typos will be caught at compile time, not after your app has already started.

## Minimal runtime

Since most of the work happens at compile time, the runtime footprint is small.
The Maud runtime library, including integration with the [Rocket] and [Actix] web frameworks, is around 100 SLoC.

[Rocket]: https://rocket.rs/
[Actix]: https://actix.rs/

## Simple deployment

There is no need to track separate template files, since all relevant code is linked into the final executable.
