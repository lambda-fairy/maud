#![feature(conservative_impl_trait, plugin)]
#![plugin(maud_macros)]

extern crate maud;

use std::fmt;

#[test]
fn if_expr() {
    for (number, &name) in (1..4).zip(["one", "two", "three"].iter()) {
        let mut s = String::new();
        html!(s, {
            @if number == 1 {
                "one"
            } @else if number == 2 {
                "two"
            } @else if number == 3 {
                "three"
            } @else {
                "oh noes"
            }
        }).unwrap();
        assert_eq!(s, name);
    }
}

#[test]
fn if_let() {
    for &(input, output) in [(Some("yay"), "yay"), (None, "oh noes")].iter() {
        let mut s = String::new();
        html!(s, {
            @if let Some(value) = input {
                ^value
            } @else {
                "oh noes"
            }
        }).unwrap();
        assert_eq!(s, output);
    }
}

#[test]
fn for_expr() {
    let ponies = ["Apple Bloom", "Scootaloo", "Sweetie Belle"];
    let mut s = String::new();
    html!(s, {
        ul @for pony in &ponies {
            li ^pony
        }
    }).unwrap();
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
        let mut s = String::new();
        html!(s, {
            @match input {
                Some(value) => {
                    div { ^value }
                },
                None => {
                    "oh noes"
                },
            }
        }).unwrap();
        assert_eq!(s, output);
    }
}

#[test]
fn match_expr_without_delims() {
    for &(input, output) in [(Some("yay"), "yay"), (None, "<span>oh noes</span>")].iter() {
        let mut s = String::new();
        html!(s, {
            @match input {
                Some(value) => ^value,
                None => span { "oh noes" },
            }
        }).unwrap();
        assert_eq!(s, output);
    }
}

#[test]
fn match_expr_with_guards() {
    for &(input, output) in [(Some(1), "one"), (None, "none"), (Some(2), "2")].iter() {
        let mut s = String::new();
        html!(s, {
            @match input {
                Some(value) if value == 1 => "one",
                Some(value) => ^value,
                None => "none",
            }
        }).unwrap();
        assert_eq!(s, output);
    }
}

#[test]
fn match_in_attribute() {
    for &(input, output) in [(1, "<span class=\"one\">1</span>"), (2, "<span class=\"two\">2</span>"), (3, "<span class=\"many\">3</span>")].iter() {
        let mut s = String::new();
        html!(s, {
            span class=@match input {
                1 => "one",
                2 => "two",
                _ => "many",
            } { ^input }
        }).unwrap();
        assert_eq!(s, output);
    }
}

#[test]
fn call() {
    fn ducks(w: &mut fmt::Write) -> fmt::Result {
        write!(w, "Ducks")
    }
    let mut s = String::new();
    let swans = |yes|
        if yes {
            |w: &mut fmt::Write| write!(w, "Swans")
        } else {
            panic!("oh noes")
        };
    html!(s, {
        @ducks
        @(|w: &mut fmt::Write| write!(w, "Geese"))
        @swans(true)
    }).unwrap();
    assert_eq!(s, "DucksGeeseSwans");
}

fn assert_cute<'a>(name: &'a str) -> impl maud::Template + 'a {
    template! {
        p {
            ^name " is the cutest"
        }
    }
}

#[test]
fn template() {
    let mut s = String::new();
    html!(s, {
        @assert_cute("Pinkie Pie")
        @assert_cute("Rarity")
    }).unwrap();
    assert_eq!(s, "<p>Pinkie Pie is the cutest</p><p>Rarity is the cutest</p>");
}
