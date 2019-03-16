use comrak::{self, ComrakOptions};
use comrak::nodes::AstNode;
use crate::Page;
use indexmap::IndexMap;
use maud::{DOCTYPE, Markup, Render, html};
use std::io;
use std::str;

struct StringWriter<'a>(&'a mut String);

impl<'a> io::Write for StringWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        str::from_utf8(buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
            .map(|s| {
                self.0.push_str(s);
                buf.len()
            })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct Comrak<'a>(&'a AstNode<'a>, &'a ComrakOptions);

impl<'a> Render for Comrak<'a> {
    fn render_to(&self, buffer: &mut String) {
        comrak::format_html(self.0, self.1, &mut StringWriter(buffer)).unwrap();
    }
}

struct ComrakText<'a>(&'a AstNode<'a>, &'a ComrakOptions);

impl<'a> Render for ComrakText<'a> {
    fn render_to(&self, buffer: &mut String) {
        comrak::format_commonmark(self.0, self.1, &mut StringWriter(buffer)).unwrap();
    }
}

crate fn main<'a>(
    options: &'a ComrakOptions,
    path: &str,
    pages: &IndexMap<&str, Page<'a>>,
) -> Markup {
    let page = &pages[path];
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title {
            @if let Some(title) = page.title {
                (ComrakText(title, options))
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
                @for (other_path, other_page) in pages {
                    @if let Some(title) = other_page.title {
                        li {
                            a href={ (other_path) ".html" } {
                                (Comrak(title, options))
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
                    a href="https://github.com/lfairy/maud" {
                        "GitHub"
                    }
                }
            }
        }

        main {
            @if let Some(title) = page.title {
                h2 {
                    (Comrak(title, options))
                }
            }
            (Comrak(page.content, options))
        }
    }
}
