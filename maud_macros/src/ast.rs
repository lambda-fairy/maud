use proc_macro::{Span, TokenStream};

#[derive(Debug)]
pub enum Markup {
    Block(Block),
    Literal {
        content: String,
        span: Span,
    },
    Symbol {
        symbol: TokenStream,
    },
    Splice {
        expr: TokenStream,
    },
    Element {
        name: TokenStream,
        attrs: Attrs,
        body: ElementBody,
    },
    Let {
        tokens: TokenStream,
    },
    Special {
        segments: Vec<Special>,
    },
    Match {
        at_span: Span,
        head: TokenStream,
        arms: Vec<MatchArm>,
        arms_span: Span,
    },
}

#[derive(Debug)]
pub struct Attrs {
    pub classes_static: Vec<ClassOrId>,
    pub classes_toggled: Vec<(ClassOrId, Toggler)>,
    pub ids: Vec<ClassOrId>,
    pub attrs: Vec<Attribute>,
}

pub type ClassOrId = TokenStream;

#[derive(Debug)]
pub enum ElementBody {
    Void { semi_span: Span },
    Block { block: Block },
}

#[derive(Debug)]
pub struct Block {
    pub markups: Vec<Markup>,
    pub outer_span: Span,
}

#[derive(Debug)]
pub struct Special {
    pub at_span: Span,
    pub head: TokenStream,
    pub body: Block,
}

#[derive(Debug)]
pub struct Attribute {
    pub name: TokenStream,
    pub attr_type: AttrType,
}

#[derive(Debug)]
pub enum AttrType {
    Normal {
        value: Markup,
    },
    Empty {
        toggler: Option<Toggler>,
    },
}

#[derive(Debug)]
pub struct Toggler {
    pub cond: TokenStream,
    pub cond_span: Span,
}

#[derive(Debug)]
pub struct MatchArm {
    pub head: TokenStream,
    pub body: Block,
}
