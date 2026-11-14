use maud::html_unchecked;

// These tests purposefully produce invalid html

#[test]
fn unchecked_accepts_non_void_element_with_semicolon() {
    let result = html_unchecked! {
        div;
        p;
    };
    assert_eq!(result.into_string(), "<div><p>");
}

#[test]
fn unchecked_accepts_void_element_with_braces() {
    let result = html_unchecked! {
        br {}
        img {}
    };
    assert_eq!(result.into_string(), "<br></br><img></img>");
}
