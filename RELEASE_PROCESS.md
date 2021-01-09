# Release process

1. Update [changelog](CHANGELOG.md)
2. Update `Cargo.toml`:
    - [`maud`](maud/Cargo.toml) (don't forget dependencies!)
    - [`maud_macros`](maud_macros/Cargo.toml)
    - [`maud_htmlescape`](maud_htmlescape/Cargo.toml) (if necessary)
3. Update `#![doc(html_root_html = "...")]`:
    - [`maud`](maud/src/lib.rs)
    - [`maud_macros`](maud_macros/src/lib.rs)
    - [`maud_htmlescape`](maud_htmlescape/src/lib.rs) (if necessary)
4. `cd docs && cargo update`
5. Commit to a new branch `release-X.Y.Z`, open a PR, fix issues, merge
6. [Create a release](https://github.com/lambda-fairy/maud/releases/new)
7. `cargo publish`
8. [Publish documentation](https://github.com/lambda-fairy/maud/actions?query=workflow%3A%22Publish+docs%22)
