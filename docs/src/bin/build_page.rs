use comrak::{
    self,
    nodes::{AstNode, NodeCodeBlock, NodeHeading, NodeHtmlBlock, NodeLink, NodeValue},
    Arena,
};
use docs::{
    page::{Page, COMRAK_OPTIONS},
    views,
};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::BufReader,
    path::Path,
    str,
};
use syntect::{
    highlighting::{Color, ThemeSet},
    html::highlighted_html_for_string,
    parsing::SyntaxSet,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 7 {
        return Err("invalid arguments".into());
    }
    build_page(&args[1], &args[2], &args[3], &args[4], &args[5], &args[6])
}

fn build_page(
    output_path: &str,
    slug: &str,
    input_path: &str,
    nav_path: &str,
    version: &str,
    hash: &str,
) -> Result<(), Box<dyn Error>> {
    let nav: Vec<(String, Option<String>)> = {
        let file = File::open(nav_path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)?
    };

    let arena = Arena::new();

    let nav = nav
        .iter()
        .filter_map(|(slug, title)| {
            title.as_ref().map(|title| {
                let title = comrak::parse_document(&arena, title, &COMRAK_OPTIONS);
                (slug.as_str(), title)
            })
        })
        .collect::<Vec<_>>();

    let page = Page::load(&arena, input_path)?;
    postprocess(page.content)?;

    let markup = views::main(slug, page, &nav, version, hash);

    fs::create_dir_all(Path::new(output_path).parent().unwrap())?;
    fs::write(output_path, markup.into_string())?;

    Ok(())
}

fn postprocess<'a>(content: &'a AstNode<'a>) -> Result<(), Box<dyn Error>> {
    lower_headings(content);
    rewrite_md_links(content);
    strip_hidden_code(content);
    highlight_code(content)?;
    Ok(())
}

fn lower_headings<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Heading(NodeHeading { level, .. }) = &mut data.value {
            *level += 1;
        }
    }
}

fn rewrite_md_links<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Link(NodeLink { url, .. }) = &mut data.value {
            if url.ends_with(".md") {
                url.truncate(url.len() - ".md".len());
                url.push_str(".html");
            }
        }
    }
}

fn strip_hidden_code<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::CodeBlock(NodeCodeBlock { info, literal, .. }) = &mut data.value {
            let info = parse_code_block_info(info);
            if !info.contains(&"rust") {
                continue;
            }
            *literal = strip_hidden_code_inner(literal);
        }
    }
}

fn strip_hidden_code_inner(literal: &str) -> String {
    let lines = literal
        .split('\n')
        .filter(|line| {
            let line = line.trim();
            line != "#" && !line.starts_with("# ")
        })
        .collect::<Vec<_>>();
    lines.join("\n")
}

fn highlight_code<'a>(root: &'a AstNode<'a>) -> Result<(), Box<dyn Error>> {
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
            let info = parse_code_block_info(info);
            let syntax = info
                .into_iter()
                .filter_map(|token| ss.find_syntax_by_token(token))
                .next()
                .unwrap_or_else(|| ss.find_syntax_plain_text());
            let mut literal = std::mem::take(literal);
            if !literal.ends_with('\n') {
                // Syntect expects a trailing newline
                literal.push('\n');
            }
            let html = highlighted_html_for_string(&literal, &ss, syntax, &theme)?;
            data.value = NodeValue::HtmlBlock(NodeHtmlBlock {
                literal: html,
                ..Default::default()
            });
        }
    }
    Ok(())
}

fn parse_code_block_info(info: &str) -> Vec<&str> {
    info.split(',').map(str::trim).collect()
}
