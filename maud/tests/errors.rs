use trybuild::TestCases;

#[test]
#[rustversion::nightly]
fn run_warnings() {
    let config = TestCases::new();
    config.compile_fail("tests/warnings/*.rs");
}

#[test]
#[rustversion::not(nightly)]
fn run_warnings() {
    let config = TestCases::new();

    config.compile_fail("tests/warnings/attribute-missing-value.rs");
    config.compile_fail("tests/warnings/class-shorthand-missing-value.rs");
    config.compile_fail("tests/warnings/dynamic-attribute-names.rs");
    config.compile_fail("tests/warnings/elements-in-attributes.rs");
    config.compile_fail("tests/warnings/let-without-block.rs");
    config.compile_fail("tests/warnings/non-closed-element.rs");
    config.compile_fail("tests/warnings-stable/*.rs");
}
