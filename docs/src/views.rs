use comrak::nodes::AstNode;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use std::str;

use crate::{
    highlight::Highlighter,
    page::{default_comrak_options, Page},
    string_writer::StringWriter,
};

pub fn main<'a>(
    slug: &str,
    page: Page<'a>,
    nav: &[(&str, &'a AstNode<'a>)],
    version: &str,
    hash: &str,
) -> Markup {
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
                    (ComrakRemovePTags(title))
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

struct Comrak<'a>(&'a AstNode<'a>);

impl Render for Comrak<'_> {
    fn render_to(&self, buffer: &mut String) {
        let highlighter = Highlighter::get();
        comrak::format_html_with_plugins(
            self.0,
            &default_comrak_options(),
            &mut StringWriter(buffer),
            &highlighter.as_plugins(),
        )
        .unwrap();
    }
}

/// Hack! The page title is wrapped in a `Paragraph` node, which introduces an
/// extra `<p>` tag that we don't want most of the time.
struct ComrakRemovePTags<'a>(&'a AstNode<'a>);

impl Render for ComrakRemovePTags<'_> {
    fn render(&self) -> Markup {
        let mut buffer = String::new();
        let highlighter = Highlighter::get();
        comrak::format_html_with_plugins(
            self.0,
            &default_comrak_options(),
            &mut StringWriter(&mut buffer),
            &highlighter.as_plugins(),
        )
        .unwrap();
        assert!(buffer.starts_with("<p>") && buffer.ends_with("</p>\n"));
        PreEscaped(
            buffer
                .trim_start_matches("<p>")
                .trim_end_matches("</p>\n")
                .to_string(),
        )
    }
}

struct ComrakText<'a>(&'a AstNode<'a>);

impl Render for ComrakText<'_> {
    fn render_to(&self, buffer: &mut String) {
        comrak::format_commonmark(self.0, &default_comrak_options(), &mut StringWriter(buffer))
            .unwrap();
    }
}
