#![feature(phase)]

extern crate maud;
#[phase(plugin)] extern crate maud_macros;

#[test]
fn it_works() {
    html! test_template("du\tcks" -23 3.14 '\n' "geese");
    let s = maud::render(test_template);
    assert_eq!(&*s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    html! template("<flim&flam>");
    let s = maud::render(template);
    assert_eq!(&*s, "&lt;flim&amp;flam&gt;");
}
