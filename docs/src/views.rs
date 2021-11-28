use comrak::nodes::AstNode;
use maud::{html, Html, ToHtml, DOCTYPE};
use std::str;

use crate::{
    page::{Page, COMRAK_OPTIONS},
    text_writer::TextWriter,
};

struct Comrak<'a>(&'a AstNode<'a>);

impl<'a> ToHtml for Comrak<'a> {
    fn html(&self, buffer: &mut Html) {
        // XSS-Safety: The input Markdown comes from docs, which are trusted.
        comrak::format_html(
            self.0,
            &COMRAK_OPTIONS,
            &mut TextWriter(buffer.as_mut_string_unchecked()),
        )
        .unwrap();
    }
}

/// Hack! Comrak wraps a single line of input in `<p>` tags, which is great in
/// general but not suitable for links in the navigation bar.
struct ComrakRemovePTags<'a>(&'a AstNode<'a>);

impl<'a> ToHtml for ComrakRemovePTags<'a> {
    fn to_html(&self) -> Html {
        let mut buffer = String::new();
        comrak::format_html(self.0, &COMRAK_OPTIONS, &mut TextWriter(&mut buffer)).unwrap();
        assert!(buffer.starts_with("<p>") && buffer.ends_with("</p>\n"));
        // XSS-Safety: The input Markdown comes from docs, which are trusted.
        Html::from_unchecked(
            buffer
                .trim_start_matches("<p>")
                .trim_end_matches("</p>\n")
                .to_string(),
        )
    }
}

struct ComrakText<'a>(&'a AstNode<'a>);

impl<'a> ToHtml for ComrakText<'a> {
    fn html(&self, buffer: &mut Html) {
        comrak::format_commonmark(self.0, &COMRAK_OPTIONS, &mut TextWriter(buffer)).unwrap();
    }
}

pub fn main<'a>(
    slug: &str,
    page: Page<'a>,
    nav: &[(&str, &'a AstNode<'a>)],
    version: &str,
    hash: &str,
) -> Html {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title {
            @if let Some(title) = page.title {
                (ComrakText(title))
                " \u{2013} "
            }
            "Maud, a macro for writing HTML"
        }
        link rel="stylesheet" href="styles.css";
        meta name="theme-color" content="#808";
        meta name="viewport" content="width=device-width";

        header {
            h1 {
                a href="." {
                    "maud"
                }
            }
        }

        nav {
            ul {
                @for &(other_slug, other_title) in nav {
                    li {
                        @if other_slug == slug {
                            b {
                                (ComrakRemovePTags(other_title))
                            }
                        } @else {
                            a href={ (other_slug) ".html" } {
                                (ComrakRemovePTags(other_title))
                            }
                        }
                    }
                }
            }
            ul {
                li {
                    a href="https://docs.rs/maud/" {
                        "API documentation"
                    }
                }
                li {
                    a href="https://github.com/lambda-fairy/maud" {
                        "GitHub"
                    }
                }
            }
        }

        main {
            @if let Some(title) = page.title {
                h2 {
                    (Comrak(title))
                }
            }
            (Comrak(page.content))
        }

        footer {
            p {
                a href={ "https://github.com/lambda-fairy/maud/tree/" (hash) } {
                    (version)
                }
            }
        }
    }
}
