use rustc_version::{version_meta, Channel};

fn main() {
    match version_meta().unwrap().channel {
        Channel::Dev | Channel::Nightly => println!("cargo:rustc-cfg=unstable"),
        Channel::Beta | Channel::Stable => {}
    }
}
