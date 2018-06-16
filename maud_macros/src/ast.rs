use proc_macro::{Span, TokenStream, TokenTree};

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
        outer_span: Span,
    },
    Element {
        name: TokenStream,
        attrs: Attrs,
        body: ElementBody,
    },
    Let {
        at_span: Span,
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

impl Markup {
    pub fn span(&self) -> Span {
        match *self {
            Markup::Block(ref block) => block.span(),
            Markup::Literal { span, .. } => span,
            Markup::Symbol { ref symbol } => span_tokens(symbol.clone()),
            Markup::Splice { outer_span, .. } => outer_span,
            Markup::Element { ref name, ref body, .. } => {
                let name_span = span_tokens(name.clone());
                name_span.join(body.span()).unwrap_or(name_span)
            },
            Markup::Let { at_span, ref tokens } => {
                at_span.join(span_tokens(tokens.clone())).unwrap_or(at_span)
            },
            Markup::Special { ref segments } => {
                join_spans(segments.iter().map(|segment| segment.span()))
            },
            Markup::Match { at_span, arms_span, .. } => {
                at_span.join(arms_span).unwrap_or(at_span)
            },
        }
    }
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

impl ElementBody {
    pub fn span(&self) -> Span {
        match *self {
            ElementBody::Void { semi_span } => semi_span,
            ElementBody::Block { ref block } => block.span(),
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub markups: Vec<Markup>,
    pub outer_span: Span,
}

impl Block {
    pub fn span(&self) -> Span {
        self.outer_span
    }
}

#[derive(Debug)]
pub struct Special {
    pub at_span: Span,
    pub head: TokenStream,
    pub body: Block,
}

impl Special {
    pub fn span(&self) -> Span {
        let body_span = self.body.span();
        self.at_span.join(body_span).unwrap_or(self.at_span)
    }
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

pub fn span_tokens<I: IntoIterator<Item=TokenTree>>(tokens: I) -> Span {
    join_spans(tokens.into_iter().map(|token| token.span()))
}

pub fn join_spans<I: IntoIterator<Item=Span>>(spans: I) -> Span {
    let mut iter = spans.into_iter();
    let mut span = match iter.next() {
        Some(span) => span,
        None => return Span::call_site(),
    };
    for new_span in iter {
        if let Some(joined) = span.join(new_span) {
            span = joined;
        }
    }
    span
}
