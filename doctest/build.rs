use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;

fn main() {
    const DOCS_DIR: &str = "../docs/content";

    // Rebuild if a chapter is added or removed
    println!("cargo:rerun-if-changed={}", DOCS_DIR);

    let mut buffer = r#"// Automatically @generated – do not edit

"#.to_string();

    for entry in fs::read_dir(DOCS_DIR).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());

        let path = entry.path();
        assert_eq!(path.extension(), Some(OsStr::new("md")));

        let path_str = path.to_str().unwrap();
        let slug_str = path.file_stem().unwrap().to_str().unwrap().replace("-", "_");

        writeln!(buffer, r#"#[doc = include_str!("{}")]"#, path_str).unwrap();
        writeln!(buffer, r#"mod {} {{ }}"#, slug_str).unwrap();
    }

    fs::write("lib.rs", buffer).unwrap();
}
