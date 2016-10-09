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
- [Changed] Switch to new splice syntax using parentheses
  [#41](https://github.com/lfairy/maud/issues/41)
- [Changed] Require parentheses around the parameter to `@call`
- [Removed] All literals must now be quoted, e.g. `"42"` not `42`


[Unreleased]: https://github.com/lfairy/maud/compare/v0.12.0...HEAD
[0.12.0]: https://github.com/lfairy/maud/compare/v0.11.1...v0.12.0
[0.11.1]: https://github.com/lfairy/maud/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/lfairy/maud/compare/v0.10.0...v0.11.0
[0.11.0]: https://github.com/lfairy/maud/compare/v0.9.2...v0.10.0
