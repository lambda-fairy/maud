use proc_macro2::{TokenStream, TokenTree};
use proc_macro_error::SpanRange;

#[derive(Debug)]
pub enum Markup {
    /// Used as a placeholder value on parse error.
    ParseError {
        span: SpanRange,
    },
    Block(Block),
    Literal {
        content: String,
        span: SpanRange,
    },
    Symbol {
        symbol: TokenStream,
    },
    Splice {
        expr: TokenStream,
        outer_span: SpanRange,
    },
    Element {
        name: TokenStream,
        attrs: Vec<Attr>,
        body: ElementBody,
    },
    Let {
        at_span: SpanRange,
        tokens: TokenStream,
    },
    Special {
        segments: Vec<Special>,
    },
    Match {
        at_span: SpanRange,
        head: TokenStream,
        arms: Vec<MatchArm>,
        arms_span: SpanRange,
    },
}

impl Markup {
    pub fn span(&self) -> SpanRange {
        match *self {
            Markup::ParseError { span } => span,
            Markup::Block(ref block) => block.span(),
            Markup::Literal { span, .. } => span,
            Markup::Symbol { ref symbol } => span_tokens(symbol.clone()),
            Markup::Splice { outer_span, .. } => outer_span,
            Markup::Element {
                ref name, ref body, ..
            } => {
                let name_span = span_tokens(name.clone());
                name_span.join_range(body.span())
            }
            Markup::Let {
                at_span,
                ref tokens,
            } => at_span.join_range(span_tokens(tokens.clone())),
            Markup::Special { ref segments } => join_ranges(segments.iter().map(Special::span)),
            Markup::Match {
                at_span, arms_span, ..
            } => at_span.join_range(arms_span),
        }
    }
}

#[derive(Debug)]
pub enum Attr {
    Class {
        dot_span: SpanRange,
        name: Markup,
        toggler: Option<Toggler>,
    },
    Id {
        hash_span: SpanRange,
        name: Markup,
    },
    Named {
        named_attr: NamedAttr,
    },
}

impl Attr {
    pub fn span(&self) -> SpanRange {
        match *self {
            Attr::Class {
                dot_span,
                ref name,
                ref toggler,
            } => {
                let name_span = name.span();
                let dot_name_span = dot_span.join_range(name_span);
                if let Some(toggler) = toggler {
                    dot_name_span.join_range(toggler.cond_span)
                } else {
                    dot_name_span
                }
            }
            Attr::Id {
                hash_span,
                ref name,
            } => {
                let name_span = name.span();
                hash_span.join_range(name_span)
            }
            Attr::Named { ref named_attr } => named_attr.span(),
        }
    }
}

#[derive(Debug)]
pub enum ElementBody {
    Void { semi_span: SpanRange },
    Block { block: Block },
}

impl ElementBody {
    pub fn span(&self) -> SpanRange {
        match *self {
            ElementBody::Void { semi_span } => semi_span,
            ElementBody::Block { ref block } => block.span(),
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub markups: Vec<Markup>,
    pub outer_span: SpanRange,
}

impl Block {
    pub fn span(&self) -> SpanRange {
        self.outer_span
    }
}

#[derive(Debug)]
pub struct Special {
    pub at_span: SpanRange,
    pub head: TokenStream,
    pub body: Block,
}

impl Special {
    pub fn span(&self) -> SpanRange {
        let body_span = self.body.span();
        self.at_span.join_range(body_span)
    }
}

#[derive(Debug)]
pub struct NamedAttr {
    pub name: TokenStream,
    pub attr_type: AttrType,
}

impl NamedAttr {
    fn span(&self) -> SpanRange {
        let name_span = span_tokens(self.name.clone());
        if let Some(attr_type_span) = self.attr_type.span() {
            name_span.join_range(attr_type_span)
        } else {
            name_span
        }
    }
}

#[derive(Debug)]
pub enum AttrType {
    Normal { value: Markup },
    Optional { toggler: Toggler },
    Empty { toggler: Option<Toggler> },
}

impl AttrType {
    fn span(&self) -> Option<SpanRange> {
        match *self {
            AttrType::Normal { ref value } => Some(value.span()),
            AttrType::Optional { ref toggler } => Some(toggler.span()),
            AttrType::Empty { ref toggler } => toggler.as_ref().map(Toggler::span),
        }
    }
}

#[derive(Debug)]
pub struct Toggler {
    pub cond: TokenStream,
    pub cond_span: SpanRange,
}

impl Toggler {
    fn span(&self) -> SpanRange {
        self.cond_span
    }
}

#[derive(Debug)]
pub struct MatchArm {
    pub head: TokenStream,
    pub body: Block,
}

pub fn span_tokens<I: IntoIterator<Item = TokenTree>>(tokens: I) -> SpanRange {
    join_ranges(tokens.into_iter().map(|s| SpanRange::single_span(s.span())))
}

pub fn join_ranges<I: IntoIterator<Item = SpanRange>>(ranges: I) -> SpanRange {
    let mut iter = ranges.into_iter();
    let first = match iter.next() {
        Some(span) => span,
        None => return SpanRange::call_site(),
    };
    let last = iter.last().unwrap_or(first);
    first.join_range(last)
}

pub fn name_to_string(name: TokenStream) -> String {
    name.into_iter().map(|token| token.to_string()).collect()
}
