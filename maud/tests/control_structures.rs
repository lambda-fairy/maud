use maud::html;

#[test]
fn if_expr() {
    for (number, &name) in (1..4).zip(["one", "two", "three"].iter()) {
        let result = html! {
            @if number == 1 {
                "one"
            } @else if number == 2 {
                "two"
            } @else if number == 3 {
                "three"
            } @else {
                "oh noes"
            }
        };
        assert_eq!(result.into_string(), name);
    }
}

#[test]
fn if_expr_in_class() {
    for &(chocolate_milk, expected) in &[
        (0, r#"<p class="empty">Chocolate milk</p>"#),
        (1, r#"<p class="full">Chocolate milk</p>"#),
    ] {
        let result = html! {
            p.@if chocolate_milk == 0 { "empty" } @else { "full" } {
                "Chocolate milk"
            }
        };
        assert_eq!(result.into_string(), expected);
    }
}

#[test]
fn if_let() {
    for &(input, output) in &[(Some("yay"), "yay"), (None, "oh noes")] {
        let result = html! {
            @if let Some(value) = input {
                (value)
            } @else {
                "oh noes"
            }
        };
        assert_eq!(result.into_string(), output);
    }
}

#[test]
fn while_expr() {
    let mut numbers = (0..3).peekable();
    let result = html! {
        ul {
            @while numbers.peek().is_some() {
                li { (numbers.next().unwrap()) }
            }
        }
    };
    assert_eq!(
        result.into_string(),
        "<ul><li>0</li><li>1</li><li>2</li></ul>"
    );
}

#[test]
fn while_let_expr() {
    let mut numbers = 0..3;

    #[allow(clippy::while_let_on_iterator)]
    let result = html! {
        ul {
            @while let Some(n) = numbers.next() {
                li { (n) }
            }
        }
    };

    assert_eq!(
        result.into_string(),
        "<ul><li>0</li><li>1</li><li>2</li></ul>"
    );
}

#[test]
fn for_expr() {
    let ponies = ["Apple Bloom", "Scootaloo", "Sweetie Belle"];
    let result = html! {
        ul {
            @for pony in &ponies {
                li { (pony) }
            }
        }
    };
    assert_eq!(
        result.into_string(),
        concat!(
            "<ul>",
            "<li>Apple Bloom</li>",
            "<li>Scootaloo</li>",
            "<li>Sweetie Belle</li>",
            "</ul>"
        )
    );
}

#[test]
fn match_expr() {
    for &(input, output) in &[(Some("yay"), "<div>yay</div>"), (None, "oh noes")] {
        let result = html! {
            @match input {
                Some(value) => {
                    div { (value) }
                },
                None => {
                    "oh noes"
                },
            }
        };
        assert_eq!(result.into_string(), output);
    }
}

#[test]
fn match_expr_without_delims() {
    for &(input, output) in &[(Some("yay"), "yay"), (None, "<span>oh noes</span>")] {
        let result = html! {
            @match input {
                Some(value) => (value),
                None => span { "oh noes" },
            }
        };
        assert_eq!(result.into_string(), output);
    }
}

#[test]
fn match_no_trailing_comma() {
    for &(input, output) in &[(Some("yay"), "yay"), (None, "<span>oh noes</span>")] {
        let result = html! {
            @match input {
                Some(value) => { (value) }
                None => span { "oh noes" }
            }
        };
        assert_eq!(result.into_string(), output);
    }
}

#[test]
fn match_expr_with_guards() {
    for &(input, output) in &[(Some(1), "one"), (None, "none"), (Some(2), "2")] {
        let result = html! {
            @match input {
                Some(value) if value % 3 == 1 => "one",
                Some(value) => (value),
                None => "none",
            }
        };
        assert_eq!(result.into_string(), output);
    }
}

#[test]
fn match_in_attribute() {
    for &(input, output) in &[
        (1, "<span class=\"one\">1</span>"),
        (2, "<span class=\"two\">2</span>"),
        (3, "<span class=\"many\">3</span>"),
    ] {
        let result = html! {
            span class=@match input {
                1 => "one",
                2 => "two",
                _ => "many",
            } { (input) }
        };
        assert_eq!(result.into_string(), output);
    }
}

#[test]
fn let_expr() {
    let result = html! {
        @let x = 42;
        "I have " (x) " cupcakes!"
    };
    assert_eq!(result.into_string(), "I have 42 cupcakes!");
}

#[test]
fn let_lexical_scope() {
    let x = 42;
    let result = html! {
        {
            @let x = 99;
            "Twilight thought I had " (x) " cupcakes, "
        }
        "but I only had " (x) "."
    };
    assert_eq!(
        result.into_string(),
        concat!("Twilight thought I had 99 cupcakes, ", "but I only had 42.")
    );
}

#[test]
fn let_type_ascription() {
    let result = html! {
        @let mut x: Box<dyn Iterator<Item=u32>> = Box::new(vec![42].into_iter());
        "I have " (x.next().unwrap()) " cupcakes!"
    };
    assert_eq!(result.into_string(), "I have 42 cupcakes!");
}
