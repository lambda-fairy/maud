use maud::{self, html, html_to, Render};

#[test]
fn html_render_to_buffer() {
    let mut buf = String::new();

    html_to! { buf,
        p { "existing" }
    };
    
    assert_eq!(buf, "<p>existing</p>");
}

#[test]
fn html_buffer_reuse() {
    let mut buf = String::new();
    html_to! { buf,
        p { "existing" }
    };
    
    html_to! { buf, 
        p { "reused" }
    };
    
    assert_eq!(buf, "<p>existing</p><p>reused</p>");
}

#[test]
fn impl_render_to_html_to() {
    struct Foo;
    impl Render for Foo {
        fn render_to(&self, buffer: &mut String) {
            html_to! { buffer,
                a { "foobar" }                
            }
        }
    }

    let rendered = html! {
        p { (Foo) }
    }.into_string();
    
    assert_eq!(rendered, "<p><a>foobar</a></p>");
}

#[test]
fn impl_render_to_html_to_use_render_in_html_to() {
    struct Foo;
    impl Render for Foo {
        fn render_to(&self, buffer: &mut String) {
            html_to! { buffer,
                a { (42) }                
            }
        }
    }

    let rendered = html! {
        p { (Foo) }
    }.into_string();
    
    assert_eq!(rendered, "<p><a>42</a></p>");
}