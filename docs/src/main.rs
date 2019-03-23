#![feature(crate_visibility_modifier)]
#![feature(proc_macro_hygiene)]

use comrak::{self, Arena, ComrakOptions};
use comrak::nodes::{AstNode, NodeCodeBlock, NodeHeading, NodeHtmlBlock, NodeLink, NodeValue};
use std::error::Error;
use std::env;
use std::fs;
use std::io;
use std::mem;
use std::path::Path;
use std::string::FromUtf8Error;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{Color, ThemeSet};
use syntect::html::highlighted_html_for_string;

mod views;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 5 && &args[1] == "build-page" {
        build_page(&args[2], &args[3], &args[4])
    } else {
        Err("invalid arguments".into())
    }
}

fn build_page(input_path: &str, output_path: &str, slug: &str) -> Result<(), Box<dyn Error>> {
    // TODO make this list dynamically generated
    const NAV: &[(&str, Option<&str>)] = &[
        ("index", None),
        ("getting-started", Some("Getting started")),
        ("basic-syntax", Some("Basic syntax")),
        ("dynamic-content", Some("Dynamic content")),
        ("partials", Some("Partials")),
        ("control-structures", Some("Control structures")),
        ("traits", Some("Traits")),
        ("web-frameworks", Some("Web frameworks")),
        ("faq", Some("FAQ")),
    ];

    fs::create_dir_all(Path::new(output_path).parent().unwrap())?;

    let arena = Arena::new();
    let options = ComrakOptions {
        ext_header_ids: Some("".to_string()),
        unsafe_: true,
        ..ComrakOptions::default()
    };

    let page = load_page(&arena, &options, input_path)?;
    let markup = views::main(&options, slug, page, &NAV);

    fs::write(output_path, markup.into_string())?;

    Ok(())
}

struct Page<'a> {
    title: Option<&'a AstNode<'a>>,
    content: &'a AstNode<'a>,
}

fn load_page<'a>(
    arena: &'a Arena<AstNode<'a>>,
    options: &ComrakOptions,
    path: impl AsRef<Path>,
) -> io::Result<Page<'a>> {
    let buffer = fs::read_to_string(path)?;
    let content = comrak::parse_document(arena, &buffer, options);

    let title = content
        .first_child()
        .filter(|node| {
            let mut data = node.data.borrow_mut();
            if let NodeValue::Heading(NodeHeading { level: 1, .. }) = data.value {
                node.detach();
                data.value = NodeValue::Document;
                true
            } else {
                false
            }
        });

    lower_headings(content);
    rewrite_md_links(content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    highlight_code(content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    Ok(Page { title, content })
}

fn lower_headings<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Heading(NodeHeading { level, .. }) = &mut data.value {
            *level += 1;
        }
    }
}

fn rewrite_md_links<'a>(root: &'a AstNode<'a>) -> Result<(), FromUtf8Error> {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Link(NodeLink { url, .. }) = &mut data.value {
            let mut url_string = String::from_utf8(mem::replace(url, Vec::new()))?;
            if url_string.ends_with(".md") {
                url_string.truncate(url_string.len() - ".md".len());
                url_string.push_str(".html");
            }
            *url = url_string.into_bytes();
        }
    }
    Ok(())
}

fn highlight_code<'a>(root: &'a AstNode<'a>) -> Result<(), FromUtf8Error> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let mut theme = ts.themes["InspiredGitHub"].clone();
    theme.settings.background = Some(Color { r: 0xee, g: 0xee, b: 0xee, a: 0xff });
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::CodeBlock(NodeCodeBlock { info, literal, ..  }) = &mut data.value {
            let info = String::from_utf8(mem::replace(info, Vec::new()))?;
            let syntax = ss.find_syntax_by_token(&info)
                .unwrap_or_else(|| ss.find_syntax_plain_text());
            let mut literal = String::from_utf8(mem::replace(literal, Vec::new()))?;
            if !literal.ends_with('\n') {
                // Syntect expects a trailing newline
                literal.push('\n');
            }
            let html = highlighted_html_for_string(&literal, &ss, syntax, &theme);
            data.value = NodeValue::HtmlBlock(NodeHtmlBlock {
                block_type: 0,
                literal: html.into_bytes(),
            });
        }
    }
    Ok(())
}
