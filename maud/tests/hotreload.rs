//! track regressions specific to the hotreload feature in maud
use maud::{html, Markup};
use maud_macros_impl::gather_html_macro_invocations;

#[test]
fn regression_match_inline_tag() {
    fn render(x: Option<usize>) -> Markup {
        html! {
            div id="main" {
                @match x {
                    Some(42) => div.green {
                        "yes! fourty! two!"
                    },
                    Some(_) => div.yellow {
                        "it's a number?"
                    },
                    None => div.red {
                        "okay."
                    },
                }
            }
        }
    }

    assert_eq!(
        render(Some(42)).into_string(),
        r#"<div id="main"><div class="green">yes! fourty! two!</div></div>"#
    );
    assert_eq!(
        render(Some(420)).into_string(),
        r#"<div id="main"><div class="yellow">it's a number?</div></div>"#
    );
    assert_eq!(
        render(None).into_string(),
        r#"<div id="main"><div class="red">okay.</div></div>"#
    );
}

#[test]
fn regression_basic() {
    let result = html! {
        "hello world"
    };

    assert_eq!(result.into_string(), "hello world");
}

#[test]
fn test_gather_html_macro_invocations() {
    let file = file!();
    let line = line!();

    let _foo = maud::html! {
        "Hello world"
    };

    assert_eq!(
        gather_html_macro_invocations(file, line)
            .unwrap()
            .to_string(),
        "\"Hello world\""
    );
}
