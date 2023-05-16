use maud::{self, html};

#[test]
fn issue_13() {
    let owned = String::from("yay");
    let _ = html! { (owned) };
    // Make sure the `html!` call didn't move it
    let _owned = owned;
}

#[test]
fn issue_21() {
    macro_rules! greet {
        () => {{
            let name = "Pinkie Pie";
            html! {
                p { "Hello, " (name) "!" }
            }
        }};
    }

    assert_eq!(greet!().into_string(), "<p>Hello, Pinkie Pie!</p>");
}

#[test]
fn issue_21_2() {
    macro_rules! greet {
        ($name:expr) => {{
            html! {
                p { "Hello, " ($name) "!" }
            }
        }};
    }

    assert_eq!(
        greet!("Pinkie Pie").into_string(),
        "<p>Hello, Pinkie Pie!</p>"
    );
}

#[test]
fn issue_23() {
    macro_rules! wrapper {
        ($($x:tt)*) => {{
            html! { $($x)* }
        }}
    }

    let name = "Lyra";
    let result = wrapper!(p { "Hi, " (name) "!" });
    assert_eq!(result.into_string(), "<p>Hi, Lyra!</p>");
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
    let result_a = html! { (r) };
    let result_b = html! { (r) };
    assert_eq!(result_a.into_string(), "pinkie");
    assert_eq!(result_b.into_string(), "pinkie");
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

    assert_eq!(html! { (Pinkie) }.into_string(), "42");
}

#[test]
fn only_display() {
    use core::fmt::Display;

    struct OnlyDisplay;
    impl Display for OnlyDisplay {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "<hello>")
        }
    }

    assert_eq!(html! { (OnlyDisplay) }.into_string(), "&lt;hello&gt;");
    assert_eq!(html! { (&OnlyDisplay) }.into_string(), "&lt;hello&gt;");
    assert_eq!(html! { (&&OnlyDisplay) }.into_string(), "&lt;hello&gt;");
    assert_eq!(html! { (&&&OnlyDisplay) }.into_string(), "&lt;hello&gt;");
    assert_eq!(html! { (&&&&OnlyDisplay) }.into_string(), "&lt;hello&gt;");
}

#[test]
fn prefer_render_over_display() {
    use core::fmt::Display;
    use maud::Render;

    struct RenderAndDisplay;
    impl Display for RenderAndDisplay {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "<display>")
        }
    }
    impl Render for RenderAndDisplay {
        fn render_to(&self, buffer: &mut String) {
            buffer.push_str("<render>");
        }
    }

    assert_eq!(html! { (RenderAndDisplay) }.into_string(), "<render>");
    assert_eq!(html! { (&RenderAndDisplay) }.into_string(), "<render>");
    assert_eq!(html! { (&&RenderAndDisplay) }.into_string(), "<render>");
    assert_eq!(html! { (&&&RenderAndDisplay) }.into_string(), "<render>");
    assert_eq!(html! { (&&&&RenderAndDisplay) }.into_string(), "<render>");

    assert_eq!(
        html! { (maud::display(RenderAndDisplay)) }.into_string(),
        "&lt;display&gt;"
    );
}

#[test]
fn default() {
    use maud::{Markup, PreEscaped};
    assert_eq!(Markup::default().0, "");
    assert_eq!(PreEscaped::<&'static str>::default().0, "");
}

#[test]
fn render_arc() {
    let arc = std::sync::Arc::new("foo");
    assert_eq!(html! { (arc) }.into_string(), "foo");
}
