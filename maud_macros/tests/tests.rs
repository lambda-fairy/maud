#![feature(plugin)]

extern crate maud;
#[plugin] #[no_link] extern crate maud_macros;

#[test]
fn it_works() {
    let template = html!("du\tcks" -23 3.14 '\n' "geese");
    let s = maud::render(template);
    assert_eq!(s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let template = html!("<flim&flam>");
    let s = maud::render(template);
    assert_eq!(s, "&lt;flim&amp;flam&gt;");
}

#[test]
fn blocks() {
    let s = maud::render(html! {
        "hello"
        {
            " ducks";
            " geese";
        }
        " swans"
    });
    assert_eq!(s, "hello ducks geese swans");
}

mod splice {
    use super::maud;  // lol

    #[test]
    fn literal() {
        let s = maud::render(html! { $"<pinkie>" });
        assert_eq!(s, "&lt;pinkie&gt;");
    }

    #[test]
    fn raw_literal() {
        let s = maud::render(html! { $$"<pinkie>" });
        assert_eq!(s, "<pinkie>");
    }

    #[test]
    fn block() {
        let s = maud::render(html! {
            ${
                let mut result = 1i32;
                for i in range(2, 11) {
                    result *= i;
                }
                result
            }
        });
        assert_eq!(s, "3628800");
    }

    static BEST_PONY: &'static str = "Pinkie Pie";

    #[test]
    fn statics() {
        let s = maud::render(html! { $BEST_PONY });
        assert_eq!(s, "Pinkie Pie");
    }

    // FIXME: See <https://github.com/rust-lang/rust/issues/15962>
    // for why this is commented out
    /*
    #[test]
    fn closure() {
        let best_pony = "Pinkie Pie";
        let s = maud::render(html! { $best_pony });
        assert_eq!(s, "Pinkie Pie");
    }
    */
}
