use trybuild::TestCases;

use std::path::PathBuf;

#[test]
fn run_warnings() {
    let config = TestCases::new();
    config.compile_fail("tests/warnings/*.rs");
}
