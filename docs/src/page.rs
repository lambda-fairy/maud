use comrak::nodes::AstNode;

pub struct Page<'a> {
    pub title: Option<&'a AstNode<'a>>,
    pub content: &'a AstNode<'a>,
}

