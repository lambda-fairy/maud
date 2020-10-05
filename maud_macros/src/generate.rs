use maud_htmlescape::Escaper;
use proc_macro2::{Delimiter, Group, Ident, Literal, Span, TokenStream, TokenTree};
use proc_macro_error::SpanRange;
use quote::quote;

use crate::ast::*;

pub fn generate(markups: Vec<Markup>, output_ident: TokenTree) -> TokenStream {
    let mut build = Builder::new(output_ident.clone());
    Generator::new(output_ident).markups(markups, &mut build);
    build.finish()
}

struct Generator {
    output_ident: TokenTree,
}

impl Generator {
    fn new(output_ident: TokenTree) -> Generator {
        Generator { output_ident }
    }

    fn builder(&self) -> Builder {
        Builder::new(self.output_ident.clone())
    }

    fn markups(&self, markups: Vec<Markup>, build: &mut Builder) {
        for markup in markups {
            self.markup(markup, build);
        }
    }

    fn markup(&self, markup: Markup, build: &mut Builder) {
        match markup {
            Markup::Block(Block {
                markups,
                outer_span,
            }) => {
                if markups
                    .iter()
                    .any(|markup| matches!(*markup, Markup::Let { .. }))
                {
                    build.push_tokens(self.block(Block {
                        markups,
                        outer_span,
                    }));
                } else {
                    self.markups(markups, build);
                }
            }
            Markup::Literal { content, .. } => build.push_escaped(&content),
            Markup::Symbol { symbol } => self.name(symbol, build),
            Markup::Splice { expr, .. } => build.push_tokens(self.splice(expr)),
            Markup::Element { name, attrs, body } => self.element(name, attrs, body, build),
            Markup::Let { tokens, .. } => build.push_tokens(tokens),
            Markup::Special { segments } => {
                for segment in segments {
                    build.push_tokens(self.special(segment));
                }
            }
            Markup::Match {
                head,
                arms,
                arms_span,
                ..
            } => {
                build.push_tokens({
                    let body = arms.into_iter().map(|arm| self.match_arm(arm)).collect();
                    let mut body = TokenTree::Group(Group::new(Delimiter::Brace, body));
                    body.set_span(arms_span.collapse());
                    quote!(#head #body)
                });
            }
        }
    }

    fn block(
        &self,
        Block {
            markups,
            outer_span,
        }: Block,
    ) -> TokenStream {
        let mut build = self.builder();
        self.markups(markups, &mut build);
        let mut block = TokenTree::Group(Group::new(Delimiter::Brace, build.finish()));
        block.set_span(outer_span.collapse());
        TokenStream::from(block)
    }

    fn splice(&self, expr: TokenStream) -> TokenStream {
        let output_ident = self.output_ident.clone();
        quote!({
            use maud::render::{RenderInternal, RenderWrapper};
            RenderWrapper(&#expr).__maud_render_to(&mut #output_ident);
        })
    }

    fn element(&self, name: TokenStream, attrs: Attrs, body: ElementBody, build: &mut Builder) {
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

    fn name(&self, name: TokenStream, build: &mut Builder) {
        let string = name
            .into_iter()
            .map(|token| token.to_string())
            .collect::<String>();
        build.push_escaped(&string);
    }

    fn attrs(&self, attrs: Attrs, build: &mut Builder) {
        for Attribute { name, attr_type } in desugar_attrs(attrs) {
            match attr_type {
                AttrType::Normal { value } => {
                    build.push_str(" ");
                    self.name(name, build);
                    build.push_str("=\"");
                    self.markup(value, build);
                    build.push_str("\"");
                }
                AttrType::Empty { toggler: None } => {
                    build.push_str(" ");
                    self.name(name, build);
                }
                AttrType::Empty {
                    toggler: Some(toggler),
                } => {
                    let head = desugar_toggler(toggler);
                    build.push_tokens({
                        let mut build = self.builder();
                        build.push_str(" ");
                        self.name(name, &mut build);
                        let body = build.finish();
                        quote!(#head { #body })
                    })
                }
            }
        }
    }

    fn special(&self, Special { head, body, .. }: Special) -> TokenStream {
        let body = self.block(body);
        quote!(#head #body)
    }

    fn match_arm(&self, MatchArm { head, body }: MatchArm) -> TokenStream {
        let body = self.block(body);
        quote!(#head #body)
    }
}

////////////////////////////////////////////////////////

fn desugar_attrs(attrs: Attrs) -> Vec<Attribute> {
    let mut classes_static = vec![];
    let mut classes_toggled = vec![];
    let mut ids = vec![];
    let mut attributes = vec![];
    for attr in attrs {
        match attr {
            Attr::Class { name, toggler, .. } => {
                if let Some(toggler) = toggler {
                    classes_toggled.push((name, toggler));
                } else {
                    classes_static.push(name);
                }
            }
            Attr::Id { name, .. } => ids.push(name),
            Attr::Attribute { attribute } => attributes.push(attribute),
        }
    }
    let classes = desugar_classes_or_ids("class", classes_static, classes_toggled);
    let ids = desugar_classes_or_ids("id", ids, vec![]);
    classes.into_iter().chain(ids).chain(attributes).collect()
}

fn desugar_classes_or_ids(
    attr_name: &'static str,
    values_static: Vec<Markup>,
    values_toggled: Vec<(Markup, Toggler)>,
) -> Option<Attribute> {
    if values_static.is_empty() && values_toggled.is_empty() {
        return None;
    }
    let mut markups = Vec::new();
    let mut leading_space = false;
    for name in values_static {
        markups.extend(prepend_leading_space(name, &mut leading_space));
    }
    for (name, toggler) in values_toggled {
        let body = Block {
            markups: prepend_leading_space(name, &mut leading_space),
            outer_span: toggler.cond_span,
        };
        let head = desugar_toggler(toggler);
        markups.push(Markup::Special {
            segments: vec![Special {
                at_span: SpanRange::call_site(),
                head,
                body,
            }],
        });
    }
    Some(Attribute {
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

fn desugar_toggler(
    Toggler {
        mut cond,
        cond_span,
    }: Toggler,
) -> TokenStream {
    // If the expression contains an opening brace `{`,
    // wrap it in parentheses to avoid parse errors
    if cond.clone().into_iter().any(is_braced_block) {
        let mut wrapped_cond = TokenTree::Group(Group::new(Delimiter::Parenthesis, cond));
        wrapped_cond.set_span(cond_span.collapse());
        cond = TokenStream::from(wrapped_cond);
    }
    quote!(if #cond)
}

fn is_braced_block(token: TokenTree) -> bool {
    matches!(token, TokenTree::Group(ref group) if group.delimiter() == Delimiter::Brace)
}

////////////////////////////////////////////////////////

struct Builder {
    output_ident: TokenTree,
    tokens: Vec<TokenTree>,
    tail: String,
}

impl Builder {
    fn new(output_ident: TokenTree) -> Builder {
        Builder {
            output_ident,
            tokens: Vec::new(),
            tail: String::new(),
        }
    }

    fn push_str(&mut self, string: &str) {
        self.tail.push_str(string);
    }

    fn push_escaped(&mut self, string: &str) {
        use std::fmt::Write;
        Escaper::new(&mut self.tail).write_str(string).unwrap();
    }

    fn push_tokens(&mut self, tokens: TokenStream) {
        self.cut();
        self.tokens.extend(tokens);
    }

    fn cut(&mut self) {
        if self.tail.is_empty() {
            return;
        }
        let push_str_expr = {
            let output_ident = self.output_ident.clone();
            let string = TokenTree::Literal(Literal::string(&self.tail));
            quote!(#output_ident.push_str(#string);)
        };
        self.tail.clear();
        self.tokens.extend(push_str_expr);
    }

    fn finish(mut self) -> TokenStream {
        self.cut();
        self.tokens.into_iter().collect()
    }
}
