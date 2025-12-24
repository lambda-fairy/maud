use std::{env, ffi::OsStr, fmt::Write as _, fs, path::Path};

fn main() {
    // Use absolute path, as the generated code may be in a different location
    let docs_dir = Path::new("../docs/content").canonicalize().unwrap();

    // Rebuild if a chapter is added or removed
    println!("cargo:rerun-if-changed={}", docs_dir.to_str().unwrap());

    let mut buffer = String::new();

    for entry in fs::read_dir(docs_dir).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());

        let path = entry.path();
        assert_eq!(path.extension(), Some(OsStr::new("md")));

        let path_str = path.to_str().unwrap();
        let slug_str = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("-", "_");

        writeln!(buffer, "#[doc = include_str!({path_str:?})]").unwrap();
        writeln!(buffer, "mod {slug_str} {{}}").unwrap();
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");
    fs::write(&dest_path, buffer).unwrap();
}
