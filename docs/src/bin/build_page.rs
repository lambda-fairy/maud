use comrak::{
    nodes::{AstNode, NodeCodeBlock, NodeHeading, NodeLink, NodeValue},
    Arena,
};
use docs::{
    page::{default_comrak_options, Page},
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
                let title = comrak::parse_document(&arena, title, &default_comrak_options());
                (slug.as_str(), title)
            })
        })
        .collect::<Vec<_>>();

    let page = Page::load(&arena, input_path)?;
    postprocess(page.content);

    let markup = views::main(slug, page, &nav, version, hash);

    fs::create_dir_all(Path::new(output_path).parent().unwrap())?;
    fs::write(output_path, markup.into_string())?;

    Ok(())
}

fn postprocess<'a>(content: &'a AstNode<'a>) {
    lower_headings(content);
    rewrite_md_links(content);
    strip_rustdoc_idioms(content);
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

fn strip_rustdoc_idioms<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::CodeBlock(NodeCodeBlock { info, literal, .. }) = &mut data.value {
            // Rustdoc uses commas, but CommonMark uses spaces
            *info = info.replace(",", " ");

            // Rustdoc uses "#" to represent hidden setup code
            if info.split_whitespace().next() == Some("rust") {
                *literal = literal
                    .split('\n')
                    .filter(|line| {
                        let line = line.trim();
                        line != "#" && !line.starts_with("# ")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            }
        }
    }
}
