use comrak::nodes::{AstNode, NodeCodeBlock, NodeHeading, NodeHtmlBlock, NodeLink, NodeValue};
use comrak::{self, Arena, ComrakOptions};
use serde_json;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::mem;
use std::path::Path;
use std::string::FromUtf8Error;
use syntect::highlighting::{Color, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

use crate::page::Page;
use crate::string_writer::StringWriter;

mod page;
mod string_writer;
mod views;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() >= 3 && &args[1] == "build-nav" && args[3..].iter().all(|arg| arg.contains(":")) {
        let entries = args[3..]
            .iter()
            .map(|arg| {
                let mut splits = arg.splitn(2, ":");
                let slug = splits.next().unwrap();
                let input_path = splits.next().unwrap();
                (slug, input_path)
            })
            .collect::<Vec<_>>();
        build_nav(&entries, &args[2])
    } else if args.len() == 6 && &args[1] == "build-page" {
        build_page(&args[2], &args[3], &args[4], &args[5])
    } else {
        Err("invalid arguments".into())
    }
}

fn build_nav(entries: &[(&str, &str)], nav_path: &str) -> Result<(), Box<dyn Error>> {
    let arena = Arena::new();
    let options = comrak_options();

    let nav = entries
        .iter()
        .map(|&(slug, input_path)| {
            let title = load_page_title(&arena, &options, input_path)?;
            Ok((slug, title))
        })
        .collect::<io::Result<Vec<_>>>()?;

    // Only write if different to avoid spurious rebuilds
    let old_string = fs::read_to_string(nav_path).unwrap_or(String::new());
    let new_string = serde_json::to_string_pretty(&nav)?;
    if old_string != new_string {
        fs::create_dir_all(Path::new(nav_path).parent().unwrap())?;
        fs::write(nav_path, new_string)?;
    }

    Ok(())
}

fn build_page(
    output_path: &str,
    slug: &str,
    input_path: &str,
    nav_path: &str,
) -> Result<(), Box<dyn Error>> {
    let nav: Vec<(String, Option<String>)> = {
        let file = File::open(nav_path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)?
    };

    let arena = Arena::new();
    let options = comrak_options();

    let nav = nav
        .iter()
        .filter_map(|(slug, title)| {
            title.as_ref().map(|title| {
                let title = comrak::parse_document(&arena, title, &options);
                (slug.as_str(), title)
            })
        })
        .collect::<Vec<_>>();

    let page = load_page(&arena, &options, input_path)?;
    let markup = views::main(&options, slug, page, &nav);

    fs::create_dir_all(Path::new(output_path).parent().unwrap())?;
    fs::write(output_path, markup.into_string())?;

    Ok(())
}

fn load_page<'a>(
    arena: &'a Arena<AstNode<'a>>,
    options: &ComrakOptions,
    path: impl AsRef<Path>,
) -> io::Result<Page<'a>> {
    let page = load_page_raw(arena, options, path)?;

    lower_headings(page.content);
    rewrite_md_links(page.content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    highlight_code(page.content).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    Ok(page)
}

fn load_page_title<'a>(
    arena: &'a Arena<AstNode<'a>>,
    options: &ComrakOptions,
    path: impl AsRef<Path>,
) -> io::Result<Option<String>> {
    let page = load_page_raw(arena, options, path)?;
    let title = page.title.map(|title| {
        let mut buffer = String::new();
        comrak::format_commonmark(title, options, &mut StringWriter(&mut buffer)).unwrap();
        buffer
    });
    Ok(title)
}

fn load_page_raw<'a>(
    arena: &'a Arena<AstNode<'a>>,
    options: &ComrakOptions,
    path: impl AsRef<Path>,
) -> io::Result<Page<'a>> {
    let buffer = fs::read_to_string(path)?;
    let content = comrak::parse_document(arena, &buffer, options);

    let title = content.first_child().filter(|node| {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Heading(NodeHeading { level: 1, .. }) = data.value {
            node.detach();
            data.value = NodeValue::Document;
            true
        } else {
            false
        }
    });

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
    theme.settings.background = Some(Color {
        r: 0xff,
        g: 0xee,
        b: 0xff,
        a: 0xff,
    });
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::CodeBlock(NodeCodeBlock { info, literal, .. }) = &mut data.value {
            let info = String::from_utf8(mem::replace(info, Vec::new()))?;
            let syntax = ss
                .find_syntax_by_token(&info)
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

fn comrak_options() -> ComrakOptions {
    ComrakOptions {
        ext_header_ids: Some("".to_string()),
        unsafe_: true,
        ..ComrakOptions::default()
    }
}
