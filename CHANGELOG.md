# Change Log

## [Unreleased]

## [0.22.0] - 2020-06-20
- [Changed] Update Actix to 2.0.0. Actix 1.0.0 is no longer supported.
  [#182](https://github.com/lambda-fairy/maud/pull/182)

## [0.21.0] - 2019-07-01

- [Added] Default tag name to `div` when a class or ID is provided
  [#173](https://github.com/lambda-fairy/maud/pull/173)
- [Changed] Maud has a new website! Check it out at <https://maud.lambda.xyz/>.
- [Changed] Update Actix to 1.0. Pre-release versions of Actix are no longer supported.
  [#176](https://github.com/lambda-fairy/maud/pull/174)
- [Fixed] Use absolute imports in generated code
  [#170](https://github.com/lambda-fairy/maud/issues/170)
- [Fixed] Use `syn` instead of `literalext`
  [#174](https://github.com/lambda-fairy/maud/pull/174)

## [0.20.0] - 2019-01-17

- [Added] Rocket 0.4 support
  [#162](https://github.com/lambda-fairy/maud/pull/162)
- [Changed] Set `Content-Type` header for Actix responses
  [#160](https://github.com/lambda-fairy/maud/pull/160)

## [0.19.0] - 2018-10-20

- [Added] Allow arbitrary syntax in class and ID shorthand
  [#128](https://github.com/lambda-fairy/maud/issues/128)
- [Added] Actix 0.7 support
  [#144](https://github.com/lambda-fairy/maud/issues/144)
- [Added] Warn on keywords without a leading `@`
  [#91](https://github.com/lambda-fairy/maud/issues/91)
- [Changed] Disallow elements that mention the same attribute twice
  [#129](https://github.com/lambda-fairy/maud/issues/129)
- [Removed] Dropped support for the `maud_lints` crate
  [66ddbca](https://github.com/lambda-fairy/maud/commit/66ddbcac986f099e309c28491c276de39340068a)
- [Fixed] Update to rustc 1.31.0-nightly (77af31408 2018-10-11)
    - The feature flags have changed again! Remove `#![feature(use_extern_macros)]` and `#![feature(proc_macro_non_items)]`, and add `#![feature(proc_macro_hygiene)]` in their place. See the [documentation][getting-started] for a working example.

## [0.18.1] - 2018-07-18

- [Fixed] Update to rustc 1.29.0-nightly (1ecf6929d 2018-07-16)
    - The `proc_macro` feature was recently stabilized ([rust-lang/rust#52081]). As a result of this change, you may get "unresolved import" errors after updating your Rust compiler. To fix this error, replace any `#![feature(proc_macro)]` in your crate with `#![feature(use_extern_macros)]`. See the [documentation][getting-started] for a working example.

[rust-lang/rust#52081]: https://github.com/rust-lang/rust/pull/52081
[getting-started]: https://maud.lambda.xyz/getting_started.html

## [0.18.0] - 2018-07-15

- [Added] Support for the Actix web framework
  [#135](https://github.com/lambda-fairy/maud/issues/135)
  [#136](https://github.com/lambda-fairy/maud/pull/136)
- [Changed] Require braces around the body of an element
  [#137](https://github.com/lambda-fairy/maud/pull/137)
- [Fixed] In a `@match` expression, allow omitting the comma on the last match arm
- [Fixed] Improved the formatting for syntax errors
- [Fixed] Update to rustc 1.28.0-nightly (5bf68db6e 2018-05-28)

## [0.17.5] - 2018-05-26

- [Fixed] Update to rustc 1.27.0-nightly (2f2a11dfc 2018-05-16)

## [0.17.4] - 2018-05-02

- [Fixed] Update to rustc 1.27.0-nightly (686d0ae13 2018-04-27)
  [#123](https://github.com/lambda-fairy/maud/issues/123)
  [#124](https://github.com/lambda-fairy/maud/pull/124)
  [#125](https://github.com/lambda-fairy/maud/issues/125)
  [#126](https://github.com/lambda-fairy/maud/pull/126)

## [0.17.3] - 2018-04-21

- [Fixed] Update to rustc 1.27.0-nightly (ac3c2288f 2018-04-18)
  [#121](https://github.com/lambda-fairy/maud/issues/121)
  [#122](https://github.com/lambda-fairy/maud/pull/122)

## [0.17.2] - 2017-11-19

- [Added] Iron 0.6 support
  [#107](https://github.com/lambda-fairy/maud/pull/107)
- [Added] Implement `Clone` and `Copy` for `PreEscaped`
  [#101](https://github.com/lambda-fairy/maud/pull/101)
- [Fixed] Allow braces in the boolean expression for a toggled class
- [Fixed] Update to rustc 1.23.0-nightly (6160040d8 2017-11-18)

## [0.17.1] - 2017-08-11

- [Fixed] "Multiple applicable items in scope" error when using `Render` trait
  [#97](https://github.com/lambda-fairy/maud/issues/97)

## [0.17.0] - 2017-08-04

- [Added] Allow terminating void elements with semicolons (`;`)
  [#96](https://github.com/lambda-fairy/maud/pull/96)
- [Changed] Update to Rocket 0.3
  [#94](https://github.com/lambda-fairy/maud/pull/94)
- [Changed] Port to new proc macro interface
  [#95](https://github.com/lambda-fairy/maud/pull/95)
- [Removed] Removed the lint plugin for now -- it'll be added back in a later version once some design issues are sorted out.
- [Fixed] Allow braces in the boolean expression for an empty attribute

## [0.16.3] - 2017-04-22

- [Fixed] Update to rustc 1.18.0-nightly (1785bca51 2017-04-21)
  [#80](https://github.com/lambda-fairy/maud/issues/80)

## [0.16.2] - 2017-03-07

- [Fixed] Update to rustc 1.17.0-nightly (b1e31766d 2017-03-03)
  [#77](https://github.com/lambda-fairy/maud/issues/77)

## [0.16.1] - 2017-02-15

- [Added] Rocket 0.2 support
  [#74](https://github.com/lambda-fairy/maud/pull/74)
- [Removed] Don't expose private `PResult` type

## [0.16.0] - 2017-02-06

- [Changed] Update to Iron 0.5
  [#70](https://github.com/lambda-fairy/maud/issues/70)
- [Fixed] Correct typo in `<!doctype html>` lint
  [#69](https://github.com/lambda-fairy/maud/issues/69)

## [0.15.0] - 2017-01-26

- [Added] Implement `Into<String>` for `Markup`
- [Added] Add a lint that suggests using the `maud::DOCTYPE` constant
  [#66](https://github.com/lambda-fairy/maud/issues/66)
- [Removed] Remove the `RenderOnce` trait
  [#68](https://github.com/lambda-fairy/maud/issues/68)
- [Fixed] Update to latest syntax extension API

## [0.14.0] - 2016-11-24

- [Added] Add a pre-defined constant for `<!DOCTYPE html>`
- [Added] Toggle a class using a boolean flag
  [#44](https://github.com/lambda-fairy/maud/issues/44)
- [Added] Let expressions
  [#57](https://github.com/lambda-fairy/maud/issues/57)
- [Changed] Toggled empty attributes now use `foo?[bar]` syntax
  [#59](https://github.com/lambda-fairy/maud/issues/59)
- [Fixed] Update to latest syntax extension API


## [0.13.0] - 2016-11-03

- [Added] Support `@while` and `@while let`
  [#55](https://github.com/lambda-fairy/maud/pull/55)
- [Changed] Change `PreEscaped` to take `AsRef<str>` instead of `Display`
  [#54](https://github.com/lambda-fairy/maud/issues/54)
- [Changed] Single quotes (`'`) are no longer escaped
- [Fixed] Update to latest syntax extension API


## [0.12.0] - 2016-10-09

- [Changed] Change `Render` and `RenderOnce` to return `Markup` instead
  [#48](https://github.com/lambda-fairy/maud/issues/48)
- [Fixed] Add a bunch of optimizations from Horrorshow
  [#46](https://github.com/lambda-fairy/maud/issues/46)


## [0.11.1] - 2016-09-25

- [Fixed] Various documentation fixes


## [0.11.0] - 2016-09-24

- [Changed] The `html!` macro now returns a `String` instead of taking a writer argument
- [Deprecated] `iron-maud` is obsolete; enable the `"iron"` feature on the `maud` crate instead
- [Removed] Remove `@call` syntax


## [0.10.0] - 2016-09-20

- [Added] Iron support
- [Added] Allow namespaces in element and attribute names
  [#38](https://github.com/lambda-fairy/maud/pull/38)
- [Changed] Switch to new splice syntax using parentheses
  [#41](https://github.com/lambda-fairy/maud/issues/41)
- [Changed] Require parentheses around the parameter to `@call`
- [Removed] All literals must now be quoted, e.g. `"42"` not `42`


## [0.9.2] - 2016-07-10

- [Fixed] Update to latest syntax extension API


## [0.9.1] - 2016-07-03

- [Fixed] Update to latest syntax extension API
- [Fixed] Silence "duplicate loop labels" warnings
  [#36](https://github.com/lambda-fairy/maud/issues/36)


## [0.9.0] - 2016-06-12

- [Added] Implement ID shorthand syntax, e.g. `div#foo`
  [#35](https://github.com/lambda-fairy/maud/issues/35)
- [Fixed] Update to latest syntax extension API


## [0.8.1] - 2016-04-27

- [Fixed] Update to latest syntax extension API


## [0.8.0] - 2016-02-28

- [Added] Add shorthand syntax for classes, e.g. `div.foo`
  [#28](https://github.com/lambda-fairy/maud/pull/28)
- [Added] Add support for `match` expressions
  [#30](https://github.com/lambda-fairy/maud/pull/30)
- [Added] Allow tuple attribute lookups (`x.0`) and identifier paths `foo::bar` in splices
  [#27](https://github.com/lambda-fairy/maud/pull/27)
  [#29](https://github.com/lambda-fairy/maud/pull/29)
- [Added] Add a `RenderOnce` trait, for when rendering a value also consumes it
  [#31](https://github.com/lambda-fairy/maud/pull/31)
- [Changed] Change symbol for special forms from `#` → `@`
  [#31](https://github.com/lambda-fairy/maud/pull/31)
- [Changed] Change symbol for splices from `$` → `^`
  [#31](https://github.com/lambda-fairy/maud/pull/31)
- [Fixed] Update to latest syntax extension API


[Unreleased]: https://github.com/lambda-fairy/maud/compare/v0.22.0...HEAD
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
