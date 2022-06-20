use comrak::{
    nodes::{AstNode, NodeHeading, NodeValue},
    Arena, ComrakOptions,
};
use std::{fs, io, path::Path, sync::LazyLock};

pub struct Page<'a> {
    pub title: Option<&'a AstNode<'a>>,
    pub content: &'a AstNode<'a>,
}

impl<'a> Page<'a> {
    pub fn load(arena: &'a Arena<AstNode<'a>>, path: impl AsRef<Path>) -> io::Result<Self> {
        let buffer = fs::read_to_string(path)?;
        let content = comrak::parse_document(arena, &buffer, &COMRAK_OPTIONS);

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

        Ok(Self { title, content })
    }
}

pub static COMRAK_OPTIONS: LazyLock<ComrakOptions> = LazyLock::new(|| {
    let mut options = ComrakOptions::default();
    options.extension.header_ids = Some("".to_string());
    options.render.unsafe_ = true;
    options
});
