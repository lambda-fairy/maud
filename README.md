# <img align="right" src="maud.png" alt="HTML5 rocks." title="HTML5 rocks."> maud 
[![Build Status](https://img.shields.io/travis/lfairy/maud.svg)](http://travis-ci.org/lfairy/maud) 
[![Cargo](https://img.shields.io/crates/v/maud.svg)](https://crates.io/crates/maud) 
[![API reference](https://docs.rs/maud/badge.svg)](https://docs.rs/maud/)

[Documentation][book] ([source][booksrc]) • [API reference][apiref] • [Change log][changelog]

Maud is an HTML template engine for Rust. It's implemented as a macro, `html!`, which compiles your markup to specialized Rust code. This unique approach makes Maud templates blazing fast, super type-safe, and easy to deploy.

Note that Maud depends on the unstable [procedural macro API][rustissue], and so requires the nightly version of Rust.

For more info on Maud, see the [official book][book].

[book]: https://maud.lambda.xyz/
[booksrc]: https://github.com/lfairy/maud/tree/master/docs
[apiref]: https://docs.rs/maud/
[changelog]: https://github.com/lfairy/maud/blob/master/CHANGELOG.md
[rustissue]: https://github.com/rust-lang/rust/issues/38356

## Stability

As of version 0.11, I am satisfied with the core syntax and semantics of the library. Development at this stage is focused on adding features and fixing bugs.

The underlying procedural macro API is still unstable though, so updating your compiler may break things. Please file an issue when this happens!
