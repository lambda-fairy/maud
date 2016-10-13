# Change Log

## [0.12.0] - 2016-10-09

- [Changed] Change `Render` and `RenderOnce` to return `Markup` instead
  [#48](https://github.com/lfairy/maud/issues/48)
- [Fixed] Add a bunch of optimizations from Horrorshow
  [#46](https://github.com/lfairy/maud/issues/46)


## [0.11.1] - 2016-09-25

- [Fixed] Various documentation fixes


## [0.11.0] - 2016-09-24

- [Changed] The `html!` macro now returns a `String` instead of taking a writer argument
- [Deprecated] `iron-maud` is obsolete; enable the `"iron"` feature on the `maud` crate instead
- [Removed] Remove `@call` syntax


## [0.10.0] - 2016-09-20

- [Added] Iron support
- [Added] Allow namespaces in element and attribute names
  [#38](https://github.com/lfairy/maud/pull/38)
- [Changed] Switch to new splice syntax using parentheses
  [#41](https://github.com/lfairy/maud/issues/41)
- [Changed] Require parentheses around the parameter to `@call`
- [Removed] All literals must now be quoted, e.g. `"42"` not `42`


## [0.9.2] - 2016-07-10

- [Fixed] Update to latest syntax extension API


## [0.9.1] - 2016-07-03

- [Fixed] Update to latest syntax extension API
- [Fixed] Silence "duplicate loop labels" warnings
  [#36](https://github.com/lfairy/maud/issues/36)


## [0.9.0] - 2016-06-12

- [Added] Implement ID shorthand syntax, e.g. `div#foo`
  [#35](https://github.com/lfairy/maud/issues/35)
- [Fixed] Update to latest syntax extension API


## [0.8.1] - 2016-04-27

- [Fixed] Update to latest syntax extension API


## [0.8.0] - 2016-02-28

- [Added] Add shorthand syntax for classes, e.g. `div.foo`
  [#28](https://github.com/lfairy/maud/pull/28)
- [Added] Add support for `match` expressions
  [#30](https://github.com/lfairy/maud/pull/30)
- [Added] Allow tuple attribute lookups (`x.0`) and identifier paths `foo::bar` in splices
  [#27](https://github.com/lfairy/maud/pull/27)
  [#29](https://github.com/lfairy/maud/pull/29)
- [Added] Add a `RenderOnce` trait, for when rendering a value also consumes it
  [#31](https://github.com/lfairy/maud/pull/31)
- [Changed] Change symbol for special forms from `#` → `@`
  [#31](https://github.com/lfairy/maud/pull/31)
- [Changed] Change symbol for splices from `$` → `^`
  [#31](https://github.com/lfairy/maud/pull/31)
- [Fixed] Update to latest syntax extension API



[Unreleased]: https://github.com/lfairy/maud/compare/v0.12.0...HEAD
[0.12.0]: https://github.com/lfairy/maud/compare/v0.11.1...v0.12.0
[0.11.1]: https://github.com/lfairy/maud/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/lfairy/maud/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/lfairy/maud/compare/v0.9.2...v0.10.0
[0.9.2]: https://github.com/lfairy/maud/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/lfairy/maud/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/lfairy/maud/compare/v0.8.1...v0.9.0
[0.8.1]: https://github.com/lfairy/maud/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/lfairy/maud/compare/v0.7.4...v0.8.0
