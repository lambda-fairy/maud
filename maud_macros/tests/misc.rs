#![feature(conservative_impl_trait, plugin)]
#![plugin(maud_macros)]

extern crate maud;

use std::fmt;

#[test]
fn html_utf8() {
    let mut buf = vec![];
    html_utf8!(buf, p "hello").unwrap();
    assert_eq!(buf, b"<p>hello</p>");
}

#[test]
fn issue_13() {
    let owned = String::from("yay");
    let mut s = String::new();
    html!(s, (owned)).unwrap();
    // Make sure the `html!` call didn't move it
    let _owned = owned;
}

#[test]
fn issue_21() {
    macro_rules! greet {
        () => ({
            let mut result = String::new();
            let name = "Pinkie Pie";
            html!(result, p { "Hello, " (name) "!" }).map(|()| result)
        })
    }

    let s = greet!().unwrap();
    assert_eq!(s, "<p>Hello, Pinkie Pie!</p>");
}

#[test]
fn issue_21_2() {
    macro_rules! greet {
        ($name:expr) => ({
            let mut result = String::new();
            html!(result, p { "Hello, " ($name) "!" }).map(|()| result)
        })
    }

    let s = greet!("Pinkie Pie").unwrap();
    assert_eq!(s, "<p>Hello, Pinkie Pie!</p>");
}

#[test]
fn issue_23() {
    macro_rules! to_string {
        ($($x:tt)*) => {{
            let mut s = String::new();
            html!(s, $($x)*).unwrap();
            s
        }}
    }

    let name = "Lyra";
    let s = to_string!(p { "Hi, " (name) "!" });
    assert_eq!(s, "<p>Hi, Lyra!</p>");
}

#[test]
fn issue_26() {
    macro_rules! to_string {
        ($($x:tt)*) => {{
            let mut s = String::new();
            html!(s, $($x)*).unwrap();
            s
        }}
    }

    let name = "Lyra";
    let s = to_string!(p { "Hi, " (name) "!" });
    assert_eq!(s, "<p>Hi, Lyra!</p>");
}

#[test]
fn issue_26_2() {
    macro_rules! to_string {
        ($($x:tt)*) => {{
            let mut s = String::new();
            html!(s, $($x)*).unwrap();
            s
        }}
    }

    let name = "Lyra";
    let s = to_string!(p { "Hi, " ("person called ".to_string() + name) "!" });
    assert_eq!(s, "<p>Hi, person called Lyra!</p>");
}

#[test]
fn render_impl() {
    struct R(&'static str);
    impl maud::Render for R {
        fn render(&self, w: &mut fmt::Write) -> fmt::Result {
            w.write_str(self.0)
        }
    }

    let mut s = String::new();
    let r = R("pinkie");
    // Since `R` is not `Copy`, this shows that Maud will auto-ref splice
    // arguments to find a `Render` impl
    html!(s, (r)).unwrap();
    html!(s, (r)).unwrap();
    assert_eq!(s, "pinkiepinkie");
}

#[test]
fn render_once_impl() {
    struct Once(String);
    impl maud::RenderOnce for Once {
        fn render_once(self, w: &mut fmt::Write) -> fmt::Result {
            w.write_str(&self.0)
        }
    }

    let mut s = String::new();
    let once = Once(String::from("pinkie"));
    html!(s, (once)).unwrap();
    assert_eq!(s, "pinkie");
}
