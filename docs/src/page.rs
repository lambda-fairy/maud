use comrak::{
    nodes::{AstNode, NodeHeading, NodeValue},
    Arena, Options,
};
use std::{fs, io, path::Path};

pub struct Page<'a> {
    pub title: Option<&'a AstNode<'a>>,
    pub content: &'a AstNode<'a>,
}

impl<'a> Page<'a> {
    pub fn load(arena: &'a Arena<AstNode<'a>>, path: impl AsRef<Path>) -> io::Result<Self> {
        let buffer = fs::read_to_string(path)?;
        let content = comrak::parse_document(arena, &buffer, &default_comrak_options());

        let title = content.first_child().filter(|node| {
            let mut data = node.data.borrow_mut();
            if let NodeValue::Heading(NodeHeading { level: 1, .. }) = data.value {
                // Split the title into a separate document
                data.value = NodeValue::Paragraph;
                let title_document = arena.alloc(NodeValue::Document.into());
                title_document.append(node);
                true
            } else {
                false
            }
        });

        Ok(Self { title, content })
    }
}

pub fn default_comrak_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.header_ids = Some("".to_string());
    options.render.unsafe_ = true;
    options
}
