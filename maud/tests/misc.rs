use maud::{self, html};

#[test]
fn issue_13() {
    let owned = String::from("yay");
    let _ = html!((owned));
    // Make sure the `html!` call didn't move it
    let _owned = owned;
}

#[test]
fn issue_21() {
    macro_rules! greet {
        () => ({
            let name = "Pinkie Pie";
            html!(p { "Hello, " (name) "!" })
        })
    }

    let s = greet!().into_string();
    assert_eq!(s, "<p>Hello, Pinkie Pie!</p>");
}

#[test]
fn issue_21_2() {
    macro_rules! greet {
        ($name:expr) => ({
            html!(p { "Hello, " ($name) "!" })
        })
    }

    let s = greet!("Pinkie Pie").into_string();
    assert_eq!(s, "<p>Hello, Pinkie Pie!</p>");
}

#[test]
fn issue_23() {
    macro_rules! wrapper {
        ($($x:tt)*) => {{
            html!($($x)*)
        }}
    }

    let name = "Lyra";
    let s = wrapper!(p { "Hi, " (name) "!" }).into_string();
    assert_eq!(s, "<p>Hi, Lyra!</p>");
}

#[test]
fn render_impl() {
    struct R(&'static str);
    impl maud::Render for R {
        fn render_to(&self, w: &mut String) {
            w.push_str(self.0);
        }
    }

    let r = R("pinkie");
    // Since `R` is not `Copy`, this shows that Maud will auto-ref splice
    // arguments to find a `Render` impl
    let s1 = html!((r)).into_string();
    let s2 = html!((r)).into_string();
    assert_eq!(s1, "pinkie");
    assert_eq!(s2, "pinkie");
}

#[test]
fn issue_97() {
    use maud::Render;

    struct Pinkie;
    impl Render for Pinkie {
        fn render(&self) -> maud::Markup {
            let x = 42;
            html! { (x) }
        }
    }

    assert_eq!(html!((Pinkie)).into_string(), "42");
}
