#![feature(plugin)]
#![allow(unstable)]

extern crate maud;
#[plugin] #[no_link] extern crate maud_macros;

#[test]
fn literals() {
    let s = html!("du\tcks" -23 3.14 '\n' "geese").render();
    assert_eq!(s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let s = html!("<flim&flam>").render();
    assert_eq!(s, "&lt;flim&amp;flam&gt;");
}

#[test]
fn semicolons() {
    let s = html! {
        "one";
        "two";
        "three";
        ;;;;;;;;;;;;;;;;;;;;;;;;
        "four";
    }.render();
    assert_eq!(s, "onetwothreefour");
}

#[test]
fn blocks() {
    let s = html! {
        "hello"
        {
            " ducks" " geese"
        }
        " swans"
    }.render();
    assert_eq!(s, "hello ducks geese swans");
}

mod elements {
    #[test]
    fn simple() {
        let s = html! {
            p { b { "pickle" } "barrel" i { "kumquat" } }
        }.render();
        assert_eq!(s, "<p><b>pickle</b>barrel<i>kumquat</i></p>");
    }

    #[test]
    fn nesting() {
        let s = html!(html body div p sup "butts").render();
        assert_eq!(s, "<html><body><div><p><sup>butts</sup></p></div></body></html>");
    }

    #[test]
    fn empty() {
        let s = html!("pinkie" br/ "pie").render();
        assert_eq!(s, "pinkie<br>pie");
    }

    #[test]
    fn attributes() {
        let s = html! {
            link rel="stylesheet" href="styles.css"/
            section id="midriff" {
                p class="hotpink" "Hello!"
            }
        }.render();
        assert_eq!(s, concat!(
                r#"<link rel="stylesheet" href="styles.css">"#,
                r#"<section id="midriff"><p class="hotpink">Hello!</p></section>"#));
    }

    #[test]
    fn empty_attributes() {
        let s = html! { div readonly=! input type="checkbox" checked=! / }.render();
        assert_eq!(s, r#"<div readonly><input type="checkbox" checked></div>"#);
    }
}

mod splices {
    #[test]
    fn literals() {
        let s = html! { $"<pinkie>" }.render();
        assert_eq!(s, "&lt;pinkie&gt;");
    }

    #[test]
    fn raw_literals() {
        let s = html! { $$"<pinkie>" }.render();
        assert_eq!(s, "<pinkie>");
    }

    #[test]
    fn blocks() {
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

    #[test]
    fn closures() {
        let best_pony = "Pinkie Pie";
        let s = html! { $best_pony }.render();
        assert_eq!(s, "Pinkie Pie");
    }

    // FIXME: See <https://github.com/rust-lang/rust/issues/16617>
    // for why this is commented out
    /*
    #[test]
    fn nested_macro_invocation() {
        let best_pony = "Pinkie Pie";
        let s = html! { $(format!("{}", best_pony)) }.render();
        assert_eq!(s, "Pinkie Pie");
    }
    */
}
