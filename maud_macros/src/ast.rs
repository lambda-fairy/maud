use proc_macro2::{Span, TokenStream, TokenTree};

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
                join_spans(segments.iter().map(Special::span))
            },
            Markup::Match { at_span, arms_span, .. } => {
                at_span.join(arms_span).unwrap_or(at_span)
            },
        }
    }
}

pub type Attrs = Vec<Attr>;

#[derive(Debug)]
pub enum Attr {
    Class {
        dot_span: Span,
        name: Markup,
        toggler: Option<Toggler>,
    },
    Id {
        hash_span: Span,
        name: Markup,
    },
    Attribute {
        attribute: Attribute,
    },
}

impl Attr {
    pub fn span(&self) -> Span {
        match *self {
            Attr::Class { dot_span, ref name, ref toggler } => {
                let name_span = name.span();
                let dot_name_span = dot_span.join(name_span).unwrap_or(dot_span);
                if let Some(toggler) = toggler {
                    dot_name_span.join(toggler.cond_span).unwrap_or(name_span)
                } else {
                    dot_name_span
                }
            },
            Attr::Id { hash_span, ref name } => {
                let name_span = name.span();
                hash_span.join(name_span).unwrap_or(hash_span)
            },
            Attr::Attribute { ref attribute } => attribute.span(),
        }
    }
}

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

impl Attribute {
    fn span(&self) -> Span {
        let name_span = span_tokens(self.name.clone());
        if let Some(attr_type_span) = self.attr_type.span() {
            name_span.join(attr_type_span).unwrap_or(name_span)
        } else {
            name_span
        }
    }
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

impl AttrType {
    fn span(&self) -> Option<Span> {
        match *self {
            AttrType::Normal { ref value } => Some(value.span()),
            AttrType::Empty { ref toggler } => toggler.as_ref().map(Toggler::span),
        }
    }
}

#[derive(Debug)]
pub struct Toggler {
    pub cond: TokenStream,
    pub cond_span: Span,
}

impl Toggler {
    fn span(&self) -> Span {
        self.cond_span
    }
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
