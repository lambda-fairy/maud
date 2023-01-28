use comrak::{self, nodes::AstNode, Arena};
use docs::{
    page::{Page, COMRAK_OPTIONS},
    string_writer::StringWriter,
};
use std::{env, error::Error, fs, io, path::Path, str};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 || !args[2..].iter().all(|arg| arg.contains(':')) {
        return Err("invalid arguments".into());
    }
    let entries = args[2..]
        .iter()
        .map(|arg| arg.split_once(':').unwrap())
        .collect::<Vec<_>>();
    build_nav(&entries, &args[1])
}

fn build_nav(entries: &[(&str, &str)], nav_path: &str) -> Result<(), Box<dyn Error>> {
    let arena = Arena::new();

    let nav = entries
        .iter()
        .map(|&(slug, input_path)| {
            let title = load_page_title(&arena, input_path)?;
            Ok((slug, title))
        })
        .collect::<io::Result<Vec<_>>>()?;

    // Only write if different to avoid spurious rebuilds
    let old_string = fs::read_to_string(nav_path).unwrap_or_default();
    let new_string = serde_json::to_string_pretty(&nav)?;
    if old_string != new_string {
        fs::create_dir_all(Path::new(nav_path).parent().unwrap())?;
        fs::write(nav_path, new_string)?;
    }

    Ok(())
}

fn load_page_title<'a>(
    arena: &'a Arena<AstNode<'a>>,
    path: impl AsRef<Path>,
) -> io::Result<Option<String>> {
    let page = Page::load(arena, path)?;
    let title = page.title.map(|title| {
        let mut buffer = String::new();
        comrak::format_commonmark(title, &COMRAK_OPTIONS, &mut StringWriter(&mut buffer)).unwrap();
        buffer
    });
    Ok(title)
}
