use std::{env, error::Error, ffi::OsStr, fmt::Write as _, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    // Use absolute path, as the generated code may be in a different location
    let docs_dir = Path::new("../docs/content").canonicalize()?;

    // Rebuild if a chapter is added or removed
    println!(
        "cargo:rerun-if-changed={}",
        docs_dir.to_str().ok_or(invalid_path(&docs_dir))?
    );

    let mut buffer = String::new();

    for entry in fs::read_dir(docs_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type()?.is_file() {
            return Err(format!("not a file: {path:?}").into());
        }
        if path.extension() != Some(OsStr::new("md")) {
            return Err(format!("not markdown: {path:?}").into());
        }

        let path_str = path.to_str().ok_or(invalid_path(&path))?;
        let slug_str = path
            .file_stem()
            .ok_or(invalid_path(&path))?
            .to_str()
            .ok_or(invalid_path(&path))?
            .replace("-", "_");

        writeln!(buffer, "#[doc = include_str!({path_str:?})]")?;
        writeln!(buffer, "mod {slug_str} {{}}")?;
    }

    let out_dir = env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("generated.rs");
    fs::write(&dest_path, buffer)?;

    Ok(())
}

fn invalid_path(path: &Path) -> String {
    format!("invalid path: {path:?}")
}
