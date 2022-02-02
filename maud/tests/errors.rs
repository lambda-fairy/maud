#[rustversion::stable(1.45.2)] // MSRV
#[test]
fn run_warnings() {
    let config = trybuild::TestCases::new();
    config.compile_fail("tests/warnings/*.rs");
}
