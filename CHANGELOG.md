# Change Log

## [Unreleased]

- Remove `html_debug!`
  [#357](https://github.com/lambda-fairy/maud/pull/357)
- Support `axum` v0.6 through `axum-core` v0.3
  [#361](https://github.com/lambda-fairy/maud/pull/361)

## [0.24.0] - 2022-08-12

- Remove blanket `Render` impl for `T: Display`
  [#320](https://github.com/lambda-fairy/maud/pull/320)
- Support `axum` v0.5 through `axum-core` v0.2
  [#325](https://github.com/lambda-fairy/maud/pull/325)
  [#337](https://github.com/lambda-fairy/maud/pull/337)
- Update to `actix-web` 4.0.
  [#331](https://github.com/lambda-fairy/maud/pull/331)
- Add a `maud::display` adapter that forwards to the `Display` impl
  [#350](https://github.com/lambda-fairy/maud/pull/350)

## [0.23.0] - 2021-11-10

- Update to support axum 0.2
  [#303](https://github.com/lambda-fairy/maud/pull/303)
- Add support for `Option<T>` attributes using the `attr=[value]` syntax.
  [#306](https://github.com/lambda-fairy/maud/pull/306)
- Update to Rust 2021
  [#309](https://github.com/lambda-fairy/maud/pull/309)
- Remove Iron support
  [#289](https://github.com/lambda-fairy/maud/pull/289)
- Disallow slashes (`/`) in void elements
  [#315](https://github.com/lambda-fairy/maud/pull/315)

## [0.22.3] - 2021-09-27

- Support `no_std` + `alloc`.
  [#278](https://github.com/lambda-fairy/maud/issues/278)
- Provide Tide support.
  [#280](https://github.com/lambda-fairy/maud/pull/280)
- Provide Axum support.
  [#284](https://github.com/lambda-fairy/maud/pull/284)

## [0.22.2] - 2021-01-09

- Don't require `?` suffix for empty attributes. The old syntax is kept for backward compatibility.
  [#238](https://github.com/lambda-fairy/maud/pull/238)
- Generalize `impl Into<String> for PreEscaped<T>` to `impl From<PreEscaped<T>> for String`.
  [#248](https://github.com/lambda-fairy/maud/pull/248)
- Use `Span::mixed_site` directly from proc-macro2
  [#254](https://github.com/lambda-fairy/maud/pull/254)

## [0.22.1] - 2020-11-02

- Stable support 🎉
  [#214](https://github.com/lambda-fairy/maud/issues/214)
- Add support for Actix Web 3.0.0. Actix Web 2.0.0 support is retained.
  [#228](https://github.com/lambda-fairy/maud/pull/228)

## [0.22.0] - 2020-06-20

- Update Actix to 2.0.0. Actix 1.0.0 is no longer supported.
  [#182](https://github.com/lambda-fairy/maud/pull/182)

## [0.21.0] - 2019-07-01

- Default tag name to `div` when a class or ID is provided
  [#173](https://github.com/lambda-fairy/maud/pull/173)
- Maud has a new website! Check it out at <https://maud.lambda.xyz/>.
- Update Actix to 1.0. Pre-release versions of Actix are no longer supported.
  [#176](https://github.com/lambda-fairy/maud/pull/174)
- Use absolute imports in generated code
  [#170](https://github.com/lambda-fairy/maud/issues/170)
- Use `syn` instead of `literalext`
  [#174](https://github.com/lambda-fairy/maud/pull/174)

## [0.20.0] - 2019-01-17

- Rocket 0.4 support
  [#162](https://github.com/lambda-fairy/maud/pull/162)
- Set `Content-Type` header for Actix responses
  [#160](https://github.com/lambda-fairy/maud/pull/160)

## [0.19.0] - 2018-10-20

- Allow arbitrary syntax in class and ID shorthand
  [#128](https://github.com/lambda-fairy/maud/issues/128)
- Actix 0.7 support
  [#144](https://github.com/lambda-fairy/maud/issues/144)
- Warn on keywords without a leading `@`
  [#91](https://github.com/lambda-fairy/maud/issues/91)
- Disallow elements that mention the same attribute twice
  [#129](https://github.com/lambda-fairy/maud/issues/129)
- Dropped support for the `maud_lints` crate
  [66ddbca](https://github.com/lambda-fairy/maud/commit/66ddbcac986f099e309c28491c276de39340068a)
- Update to rustc 1.31.0-nightly (77af31408 2018-10-11)
    - The feature flags have changed again! Remove `#![feature(use_extern_macros)]` and `#![feature(proc_macro_non_items)]`, and add `#![feature(proc_macro_hygiene)]` in their place. See the [documentation][getting-started] for a working example.

## [0.18.1] - 2018-07-18

- Update to rustc 1.29.0-nightly (1ecf6929d 2018-07-16)
    - The `proc_macro` feature was recently stabilized ([rust-lang/rust#52081]). As a result of this change, you may get "unresolved import" errors after updating your Rust compiler. To fix this error, replace any `#![feature(proc_macro)]` in your crate with `#![feature(use_extern_macros)]`. See the [documentation][getting-started] for a working example.

[rust-lang/rust#52081]: https://github.com/rust-lang/rust/pull/52081
[getting-started]: https://maud.lambda.xyz/getting_started.html

## [0.18.0] - 2018-07-15

- Support for the Actix web framework
  [#135](https://github.com/lambda-fairy/maud/issues/135)
  [#136](https://github.com/lambda-fairy/maud/pull/136)
- Require braces around the body of an element
  [#137](https://github.com/lambda-fairy/maud/pull/137)
- In a `@match` expression, allow omitting the comma on the last match arm
- Improved the formatting for syntax errors
- Update to rustc 1.28.0-nightly (5bf68db6e 2018-05-28)

## [0.17.5] - 2018-05-26

- Update to rustc 1.27.0-nightly (2f2a11dfc 2018-05-16)

## [0.17.4] - 2018-05-02

- Update to rustc 1.27.0-nightly (686d0ae13 2018-04-27)
  [#123](https://github.com/lambda-fairy/maud/issues/123)
  [#124](https://github.com/lambda-fairy/maud/pull/124)
  [#125](https://github.com/lambda-fairy/maud/issues/125)
  [#126](https://github.com/lambda-fairy/maud/pull/126)

## [0.17.3] - 2018-04-21

- Update to rustc 1.27.0-nightly (ac3c2288f 2018-04-18)
  [#121](https://github.com/lambda-fairy/maud/issues/121)
  [#122](https://github.com/lambda-fairy/maud/pull/122)

## [0.17.2] - 2017-11-19

- Iron 0.6 support
  [#107](https://github.com/lambda-fairy/maud/pull/107)
- Implement `Clone` and `Copy` for `PreEscaped`
  [#101](https://github.com/lambda-fairy/maud/pull/101)
- Allow braces in the boolean expression for a toggled class
- Update to rustc 1.23.0-nightly (6160040d8 2017-11-18)

## [0.17.1] - 2017-08-11

- "Multiple applicable items in scope" error when using `Render` trait
  [#97](https://github.com/lambda-fairy/maud/issues/97)

## [0.17.0] - 2017-08-04

- Allow terminating void elements with semicolons (`;`)
  [#96](https://github.com/lambda-fairy/maud/pull/96)
- Update to Rocket 0.3
  [#94](https://github.com/lambda-fairy/maud/pull/94)
- Port to new proc macro interface
  [#95](https://github.com/lambda-fairy/maud/pull/95)
- Removed the lint plugin for now -- it'll be added back in a later version once some design issues are sorted out.
- Allow braces in the boolean expression for an empty attribute

## [0.16.3] - 2017-04-22

- Update to rustc 1.18.0-nightly (1785bca51 2017-04-21)
  [#80](https://github.com/lambda-fairy/maud/issues/80)

## [0.16.2] - 2017-03-07

- Update to rustc 1.17.0-nightly (b1e31766d 2017-03-03)
  [#77](https://github.com/lambda-fairy/maud/issues/77)

## [0.16.1] - 2017-02-15

- Rocket 0.2 support
  [#74](https://github.com/lambda-fairy/maud/pull/74)
- Don't expose private `PResult` type

## [0.16.0] - 2017-02-06

- Update to Iron 0.5
  [#70](https://github.com/lambda-fairy/maud/issues/70)
- Correct typo in `<!doctype html>` lint
  [#69](https://github.com/lambda-fairy/maud/issues/69)

## [0.15.0] - 2017-01-26

- Implement `Into<String>` for `Markup`
- Add a lint that suggests using the `maud::DOCTYPE` constant
  [#66](https://github.com/lambda-fairy/maud/issues/66)
- [Removed] Remove the `RenderOnce` trait
  [#68](https://github.com/lambda-fairy/maud/issues/68)
- Update to latest syntax extension API

## [0.14.0] - 2016-11-24

- Add a pre-defined constant for `<!DOCTYPE html>`
- Toggle a class using a boolean flag
  [#44](https://github.com/lambda-fairy/maud/issues/44)
- Let expressions
  [#57](https://github.com/lambda-fairy/maud/issues/57)
- Toggled empty attributes now use `foo?[bar]` syntax
  [#59](https://github.com/lambda-fairy/maud/issues/59)
- Update to latest syntax extension API


## [0.13.0] - 2016-11-03

- Support `@while` and `@while let`
  [#55](https://github.com/lambda-fairy/maud/pull/55)
- Change `PreEscaped` to take `AsRef<str>` instead of `Display`
  [#54](https://github.com/lambda-fairy/maud/issues/54)
- Single quotes (`'`) are no longer escaped
- Update to latest syntax extension API


## [0.12.0] - 2016-10-09

- Change `Render` and `RenderOnce` to return `Markup` instead
  [#48](https://github.com/lambda-fairy/maud/issues/48)
- Add a bunch of optimizations from Horrorshow
  [#46](https://github.com/lambda-fairy/maud/issues/46)


## [0.11.1] - 2016-09-25

- Various documentation fixes


## [0.11.0] - 2016-09-24

- The `html!` macro now returns a `String` instead of taking a writer argument
- `iron-maud` is obsolete; enable the `"iron"` feature on the `maud` crate instead
- Remove `@call` syntax


## [0.10.0] - 2016-09-20

- Iron support
- Allow namespaces in element and attribute names
  [#38](https://github.com/lambda-fairy/maud/pull/38)
- Switch to new splice syntax using parentheses
  [#41](https://github.com/lambda-fairy/maud/issues/41)
- Require parentheses around the parameter to `@call`
- All literals must now be quoted, e.g. `"42"` not `42`


## [0.9.2] - 2016-07-10

- Update to latest syntax extension API


## [0.9.1] - 2016-07-03

- Update to latest syntax extension API
- Silence "duplicate loop labels" warnings
  [#36](https://github.com/lambda-fairy/maud/issues/36)


## [0.9.0] - 2016-06-12

- Implement ID shorthand syntax, e.g. `div#foo`
  [#35](https://github.com/lambda-fairy/maud/issues/35)
- Update to latest syntax extension API


## [0.8.1] - 2016-04-27

- Update to latest syntax extension API


## [0.8.0] - 2016-02-28

- Add shorthand syntax for classes, e.g. `div.foo`
  [#28](https://github.com/lambda-fairy/maud/pull/28)
- Add support for `match` expressions
  [#30](https://github.com/lambda-fairy/maud/pull/30)
- Allow tuple attribute lookups (`x.0`) and identifier paths `foo::bar` in splices
  [#27](https://github.com/lambda-fairy/maud/pull/27)
  [#29](https://github.com/lambda-fairy/maud/pull/29)
- Add a `RenderOnce` trait, for when rendering a value also consumes it
  [#31](https://github.com/lambda-fairy/maud/pull/31)
- Change symbol for special forms from `#` → `@`
  [#31](https://github.com/lambda-fairy/maud/pull/31)
- Change symbol for splices from `$` → `^`
  [#31](https://github.com/lambda-fairy/maud/pull/31)
- Update to latest syntax extension API


[Unreleased]: https://github.com/lambda-fairy/maud/compare/v0.24.0...HEAD
[0.24.0]: https://github.com/lambda-fairy/maud/compare/v0.23.0...v0.24.0
[0.23.0]: https://github.com/lambda-fairy/maud/compare/v0.22.3...v0.23.0
[0.22.3]: https://github.com/lambda-fairy/maud/compare/v0.22.2...v0.22.3
[0.22.2]: https://github.com/lambda-fairy/maud/compare/v0.22.1...v0.22.2
[0.22.1]: https://github.com/lambda-fairy/maud/compare/v0.22.0...v0.22.1
[0.22.0]: https://github.com/lambda-fairy/maud/compare/v0.21.0...v0.22.0
[0.21.0]: https://github.com/lambda-fairy/maud/compare/v0.20.0...v0.21.0
[0.20.0]: https://github.com/lambda-fairy/maud/compare/v0.19.0...v0.20.0
[0.19.0]: https://github.com/lambda-fairy/maud/compare/v0.18.1...v0.19.0
[0.18.1]: https://github.com/lambda-fairy/maud/compare/v0.18.0...v0.18.1
[0.18.0]: https://github.com/lambda-fairy/maud/compare/v0.17.5...v0.18.0
[0.17.5]: https://github.com/lambda-fairy/maud/compare/v0.17.4...v0.17.5
[0.17.4]: https://github.com/lambda-fairy/maud/compare/v0.17.3...v0.17.4
[0.17.3]: https://github.com/lambda-fairy/maud/compare/v0.17.2...v0.17.3
[0.17.2]: https://github.com/lambda-fairy/maud/compare/v0.17.1...v0.17.2
[0.17.1]: https://github.com/lambda-fairy/maud/compare/v0.17.0...v0.17.1
[0.17.0]: https://github.com/lambda-fairy/maud/compare/v0.16.3...v0.17.0
[0.16.3]: https://github.com/lambda-fairy/maud/compare/v0.16.2...v0.16.3
[0.16.2]: https://github.com/lambda-fairy/maud/compare/v0.16.1...v0.16.2
[0.16.1]: https://github.com/lambda-fairy/maud/compare/v0.16.0...v0.16.1
[0.16.0]: https://github.com/lambda-fairy/maud/compare/v0.15.0...v0.16.0
[0.15.0]: https://github.com/lambda-fairy/maud/compare/v0.14.0...v0.15.0
[0.14.0]: https://github.com/lambda-fairy/maud/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/lambda-fairy/maud/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/lambda-fairy/maud/compare/v0.11.1...v0.12.0
[0.11.1]: https://github.com/lambda-fairy/maud/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/lambda-fairy/maud/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/lambda-fairy/maud/compare/v0.9.2...v0.10.0
[0.9.2]: https://github.com/lambda-fairy/maud/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/lambda-fairy/maud/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/lambda-fairy/maud/compare/v0.8.1...v0.9.0
[0.8.1]: https://github.com/lambda-fairy/maud/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/lambda-fairy/maud/compare/v0.7.4...v0.8.0
