#![feature(plugin)]
#![allow(unstable)]

extern crate maud;
#[plugin] #[no_link] extern crate maud_macros;

#[test]
fn it_works() {
    let s = html!("du\tcks" -23 3.14 '\n' "geese").render();
    assert_eq!(s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let s = html!("<flim&flam>").render();
    assert_eq!(s, "&lt;flim&amp;flam&gt;");
}

#[test]
fn blocks() {
    let s = html! {
        "hello"
        {
            " ducks";
            " geese";
        }
        " swans"
    }.render();
    assert_eq!(s, "hello ducks geese swans");
}

mod splice {
    #[test]
    fn literal() {
        let s = html! { $"<pinkie>" }.render();
        assert_eq!(s, "&lt;pinkie&gt;");
    }

    #[test]
    fn raw_literal() {
        let s = html! { $$"<pinkie>" }.render();
        assert_eq!(s, "<pinkie>");
    }

    #[test]
    fn block() {
        let s = html! {
            ${
                let mut result = 1i32;
                for i in range(2, 11) {
                    result *= i;
                }
                result
            }
        }.render();
        assert_eq!(s, "3628800");
    }

    static BEST_PONY: &'static str = "Pinkie Pie";

    #[test]
    fn statics() {
        let s = html! { $BEST_PONY }.render();
        assert_eq!(s, "Pinkie Pie");
    }

    // FIXME: See <https://github.com/rust-lang/rust/issues/15962>
    // for why this is commented out
    /*
    #[test]
    fn closure() {
        let best_pony = "Pinkie Pie";
        let s = html! { $best_pony }.render();
        assert_eq!(s, "Pinkie Pie");
    }
    */
}
