#![feature(phase)]

extern crate maud;
#[phase(plugin)] extern crate maud_macros;

#[test]
fn it_works() {
    let template = html!("du\tcks" -23 3.14 '\n' "geese");
    let s = maud::render(template);
    assert_eq!(&*s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let template = html!("<flim&flam>");
    let s = maud::render(template);
    assert_eq!(&*s, "&lt;flim&amp;flam&gt;");
}
