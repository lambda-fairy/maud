extern crate compiletest_rs;

use std::path::PathBuf;

#[test]
fn run_warnings() {
    let mut config = compiletest_rs::Config::default();

    config.mode = compiletest_rs::common::Mode::Ui;
    config.src_base = PathBuf::from("tests/warnings");
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path
    config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

    compiletest_rs::run_tests(&config);
}
