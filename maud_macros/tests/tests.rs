#![feature(phase)]

extern crate maud;
#[phase(plugin)] extern crate maud_macros;

#[test]
fn it_works() {
    let mut buf = vec![];
    html! test_template("du\tcks" -23 3.14 '\n' "geese");
    test_template(&mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert_eq!(&*s, "du\tcks-233.14\ngeese");
}
