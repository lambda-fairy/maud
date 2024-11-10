use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use proc_macro_error::SpanRange;
use quote::quote;

use crate::{ast::*, escape};

pub fn generate(markups: Vec<Markup>) -> TokenStream {
    let mut build = RuntimeBuilder::new();
    RuntimeGenerator::new().markups(markups, &mut build);
    build.finish()
}

pub fn format_str(markups: Vec<Markup>) -> String {
    let mut build = RuntimeBuilder::new();
    RuntimeGenerator::new().markups(markups, &mut build);
    build.format_str()
}

struct RuntimeGenerator {}

impl RuntimeGenerator {
    fn new() -> RuntimeGenerator {
        RuntimeGenerator {}
    }

    fn builder(&self) -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    fn markups(&self, markups: Vec<Markup>, build: &mut RuntimeBuilder) {
        for markup in markups {
            self.markup(markup, build);
        }
    }

    fn markup(&self, markup: Markup, build: &mut RuntimeBuilder) {
        match markup {
            Markup::Block(Block { markups, .. }) => {
                for markup in markups {
                    self.markup(markup, build);
                }
            }
            Markup::Literal { content, .. } => build.push_escaped(&content),
            Markup::Symbol { symbol } => build.push_str(&symbol.to_string()),
            Markup::Splice { expr, .. } => build.push_format_arg(expr),
            Markup::Element { name, attrs, body } => self.element(name, attrs, body, build),
            Markup::Let { tokens, .. } => build.push_format_arg(tokens),
            Markup::Special { .. } => {} // TODO
            Markup::Match { .. } => {} // TODO
        }
    }

    fn element(&self, name: TokenStream, attrs: Vec<Attr>, body: ElementBody, build: &mut RuntimeBuilder) {
        build.push_str("<");
        self.name(name.clone(), build);
        self.attrs(attrs, build);
        build.push_str(">");
        if let ElementBody::Block { block } = body {
            self.markups(block.markups, build);
            build.push_str("</");
            self.name(name, build);
            build.push_str(">");
        }
    }

    fn name(&self, name: TokenStream, build: &mut RuntimeBuilder) {
        build.push_escaped(&name_to_string(name));
    }

    fn attrs(&self, attrs: Vec<Attr>, build: &mut RuntimeBuilder) {
        for NamedAttr { name, attr_type } in desugar_attrs(attrs) {
            match attr_type {
                AttrType::Normal { value } => {
                    build.push_str(" ");
                    self.name(name, build);
                    build.push_str("=\"");
                    self.markup(value, build);
                    build.push_str("\"");
                }
                AttrType::Optional { .. } => {}
                AttrType::Empty { toggler: None } => {
                    build.push_str(" ");
                    self.name(name, build);
                }
                AttrType::Empty { .. } => {}
            }
        }
    }
}

////////////////////////////////////////////////////////

fn desugar_attrs(attrs: Vec<Attr>) -> Vec<NamedAttr> {
    let mut classes_static = vec![];
    let mut classes_toggled = vec![];
    let mut ids = vec![];
    let mut named_attrs = vec![];
    for attr in attrs {
        match attr {
            Attr::Class {
                name,
                toggler: Some(toggler),
                ..
            } => classes_toggled.push((name, toggler)),
            Attr::Class {
                name,
                toggler: None,
                ..
            } => classes_static.push(name),
            Attr::Id { name, .. } => ids.push(name),
            Attr::Named { named_attr } => named_attrs.push(named_attr),
        }
    }
    let classes = desugar_classes_or_ids("class", classes_static, classes_toggled);
    let ids = desugar_classes_or_ids("id", ids, vec![]);
    classes.into_iter().chain(ids).chain(named_attrs).collect()
}

fn desugar_classes_or_ids(
    attr_name: &'static str,
    values_static: Vec<Markup>,
    values_toggled: Vec<(Markup, Toggler)>,
) -> Option<NamedAttr> {
    if values_static.is_empty() && values_toggled.is_empty() {
        return None;
    }
    let mut markups = Vec::new();
    let mut leading_space = false;
    for name in values_static {
        markups.extend(prepend_leading_space(name, &mut leading_space));
    }
    for (name, Toggler { cond, cond_span }) in values_toggled {
        let body = Block {
            markups: prepend_leading_space(name, &mut leading_space),
            // TODO: is this correct?
            outer_span: cond_span,
        };
        markups.push(Markup::Special {
            segments: vec![Special {
                at_span: SpanRange::call_site(),
                head: quote!(if (#cond)),
                body,
            }],
        });
    }
    Some(NamedAttr {
        name: TokenStream::from(TokenTree::Ident(Ident::new(attr_name, Span::call_site()))),
        attr_type: AttrType::Normal {
            value: Markup::Block(Block {
                markups,
                outer_span: SpanRange::call_site(),
            }),
        },
    })
}

fn prepend_leading_space(name: Markup, leading_space: &mut bool) -> Vec<Markup> {
    let mut markups = Vec::new();
    if *leading_space {
        markups.push(Markup::Literal {
            content: " ".to_owned(),
            span: name.span(),
        });
    }
    *leading_space = true;
    markups.push(name);
    markups
}

////////////////////////////////////////////////////////

struct RuntimeBuilder {
    tokens: Vec<TokenTree>,
    format_str: String,
    arg_track: u32,
}

impl RuntimeBuilder {
    fn new() -> RuntimeBuilder {
        RuntimeBuilder {
            tokens: Vec::new(),
            format_str: String::new(),
            arg_track: 0,
        }
    }

    fn push_str(&mut self, string: &str) {
        self.format_str.push_str(string);
    }

    fn push_escaped(&mut self, string: &str) {
        escape::escape_to_string(string, &mut self.format_str);
    }

    fn push_format_arg(&mut self, tokens_expr: TokenStream) {
        let arg_track = self.arg_track.to_string();
        self.tokens.extend(quote! {
            vars.insert(#arg_track, { #tokens_expr }.into());
        });
        self.arg_track = self.arg_track + 1;
        self.format_str.push_str(&format!("{{{}}}", arg_track));
    }

    fn format_str(&self) -> String {
        self.format_str.clone()
    }

    fn finish(self) -> TokenStream {
        let tokens = self.tokens.into_iter().collect::<TokenStream>();
        quote! {
            #tokens
        }.into()
    }
}
