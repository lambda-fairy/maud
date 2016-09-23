#![feature(conservative_impl_trait, plugin)]
#![plugin(maud_macros)]

extern crate maud;

#[test]
fn if_expr() {
    for (number, &name) in (1..4).zip(["one", "two", "three"].iter()) {
        let s = html! {
            @if number == 1 {
                "one"
            } @else if number == 2 {
                "two"
            } @else if number == 3 {
                "three"
            } @else {
                "oh noes"
            }
        }.into_string();
        assert_eq!(s, name);
    }
}

#[test]
fn if_let() {
    for &(input, output) in [(Some("yay"), "yay"), (None, "oh noes")].iter() {
        let s = html! {
            @if let Some(value) = input {
                (value)
            } @else {
                "oh noes"
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn for_expr() {
    let ponies = ["Apple Bloom", "Scootaloo", "Sweetie Belle"];
    let s = html! {
        ul @for pony in &ponies {
            li (pony)
        }
    }.into_string();
    assert_eq!(s, concat!(
            "<ul>",
            "<li>Apple Bloom</li>",
            "<li>Scootaloo</li>",
            "<li>Sweetie Belle</li>",
            "</ul>"));
}

#[test]
fn match_expr() {
    for &(input, output) in [(Some("yay"), "<div>yay</div>"), (None, "oh noes")].iter() {
        let s = html! {
            @match input {
                Some(value) => {
                    div (value)
                },
                None => {
                    "oh noes"
                },
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn match_expr_without_delims() {
    for &(input, output) in [(Some("yay"), "yay"), (None, "<span>oh noes</span>")].iter() {
        let s = html! {
            @match input {
                Some(value) => (value),
                None => span "oh noes",
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn match_expr_with_guards() {
    for &(input, output) in [(Some(1), "one"), (None, "none"), (Some(2), "2")].iter() {
        let s = html! {
            @match input {
                Some(value) if value == 1 => "one",
                Some(value) => (value),
                None => "none",
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn match_in_attribute() {
    for &(input, output) in [(1, "<span class=\"one\">1</span>"), (2, "<span class=\"two\">2</span>"), (3, "<span class=\"many\">3</span>")].iter() {
        let s = html! {
            span class=@match input {
                1 => "one",
                2 => "two",
                _ => "many",
            } { (input) }
        }.into_string();
        assert_eq!(s, output);
    }
}

fn assert_cute<'a>(name: &'a str) -> impl FnOnce(&mut String) + 'a {
    move |s: &mut String| s.push_str(&html! {
        p {
            (name) " is the cutest"
        }
    }.into_string())
}

#[test]
fn call() {
    let s = html! {
        @call (assert_cute("Pinkie Pie"))
        @call (assert_cute("Rarity"))
    }.into_string();
    assert_eq!(s, "<p>Pinkie Pie is the cutest</p><p>Rarity is the cutest</p>");
}
