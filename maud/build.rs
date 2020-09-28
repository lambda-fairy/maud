use rustc_version::{version_meta, Channel};

fn main() {
    match version_meta().map(|v| v.channel).unwrap_or(Channel::Stable) {
        Channel::Dev | Channel::Nightly => println!("cargo:rustc-cfg=unstable"),
        Channel::Beta | Channel::Stable => {}
    }
}
