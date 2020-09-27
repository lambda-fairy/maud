// Diagnostics look slightly different on stable Rust
#![cfg(unstable)]

use trybuild::TestCases;

#[test]
fn run_warnings() {
    let config = TestCases::new();
    config.compile_fail("tests/warnings/*.rs");
}
