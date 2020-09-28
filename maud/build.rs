fn main() {
    if version_check::is_feature_flaggable() == Some(true) {
        println!("cargo:rustc-cfg=unstable");
    }
}
