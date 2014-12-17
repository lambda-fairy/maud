#![feature(phase)]

extern crate htmlthing;
#[phase(plugin)] extern crate htmlthing_macros;

#[test]
fn it_works() {
    let s = html!(ducks);
    assert_eq!(s, "ducks");
}
