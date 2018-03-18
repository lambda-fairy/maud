use maud_htmlescape::Escaper;
use proc_macro::{Delimiter, Literal, quote, Spacing, Span, Term, TokenNode, TokenStream, TokenTree};

use ast::*;

pub fn generate(markups: Vec<Markup>, output_ident: TokenTree) -> TokenStream {
    let mut tail = Tail::new(output_ident.clone());
    let result = Generator::new(output_ident).markups(markups, &mut tail);
    tail.finish(result)
}

struct Generator {
    output_ident: TokenTree,
}

impl Generator {
    fn new(output_ident: TokenTree) -> Generator {
        Generator { output_ident }
    }

    fn markups(&self, markups: Vec<Markup>, tail: &mut Tail) -> TokenStream {
        markups.into_iter().map(|markup| self.markup(markup, tail)).collect()
    }

    fn markup(&self, markup: Markup, tail: &mut Tail) -> TokenStream {
        match markup {
            Markup::Block(Block { markups, span }) => {
                if markups.iter().any(|markup| matches!(*markup, Markup::Let { .. })) {
                    tail.cut_once(move |tail| self.block(Block { markups, span }, tail))
                } else {
                    self.markups(markups, tail)
                }
            },
            Markup::Literal { content, span } => self.literal(&content, span, tail),
            Markup::Symbol { symbol } => self.symbol(symbol, tail),
            Markup::Splice { expr } => self.splice(expr, tail),
            Markup::Element { name, attrs, body } => self.element(name, attrs, body, tail),
            Markup::Let { tokens } => tail.cut_once(|_| tokens),
            Markup::If { segments } => {
                tail.cut_many(move |kitsune| {
                    segments
                        .into_iter()
                        .map(|segment| self.special(segment, kitsune.fork()))
                        .collect()
                })
            },
            Markup::Special(special) => tail.cut_once(move |tail| self.special(special, tail)),
            Markup::Match { head, arms, arms_span } => {
                tail.cut_many(move |kitsune| {
                    let body = arms
                        .into_iter()
                        .map(|arm| self.special(arm, kitsune.fork()))
                        .collect();
                    let body = TokenTree {
                        kind: TokenNode::Group(Delimiter::Brace, body),
                        span: arms_span,
                    };
                    quote!($head $body)
                })
            },
        }
    }

    fn block(&self, Block { markups, span }: Block, mut tail: Tail) -> TokenStream {
        let markups = self.markups(markups, &mut tail);
        TokenStream::from(TokenTree {
            kind: TokenNode::Group(Delimiter::Brace, tail.finish(markups)),
            span,
        })
    }

    fn literal(&self, content: &str, span: Span, tail: &mut Tail) -> TokenStream {
        tail.push_escaped(content);
        let marker = TokenTree {
            kind: TokenNode::Literal(Literal::string(content)),
            span,
        };
        quote!(maud::marker::literal(&[$marker]);)
    }

    fn symbol(&self, symbol: TokenStream, tail: &mut Tail) -> TokenStream {
        let marker = self.name(symbol, tail);
        quote!(maud::marker::literal($marker);)
    }

    fn splice(&self, expr: TokenStream, tail: &mut Tail) -> TokenStream {
        let output_ident = self.output_ident.clone();
        tail.cut_once(move |_| quote!({
            // Create a local trait alias so that autoref works
            trait Render: maud::Render {
                fn __maud_render_to(&self, output_ident: &mut String) {
                    maud::Render::render_to(self, output_ident);
                }
            }
            impl<T: maud::Render> Render for T {}
            $expr.__maud_render_to(&mut $output_ident);
        }))
    }

    fn element(
        &self,
        name: TokenStream,
        attrs: Attrs,
        body: Option<Box<Markup>>,
        tail: &mut Tail,
    ) -> TokenStream {
        tail.push_str("<");
        let name_marker = self.name(name.clone(), tail);
        let attrs_marker = self.attrs(attrs, tail);
        tail.push_str(">");
        let body_marker = if let Some(body) = body {
            let body_marker = self.markup(*body, tail);
            tail.push_str("</");
            for token in name {
                tail.push_str(&token.to_string());
            }
            tail.push_str(">");
            body_marker
        } else {
            TokenStream::empty()
        };
        quote!(maud::marker::element($name_marker, { $attrs_marker }, { $body_marker });)
    }

    fn name(&self, name: TokenStream, tail: &mut Tail) -> TokenStream {
        let mut markers = Vec::new();
        for token in name {
            let fragment = token.to_string();
            markers.push(TokenTree {
                kind: TokenNode::Literal(Literal::string(&fragment)),
                span: token.span,
            });
            tail.push_escaped(&fragment);
            markers.push(TokenTree {
                kind: TokenNode::Op(',', Spacing::Alone),
                span: token.span,
            });
        }
        let markers = markers.into_iter().collect::<TokenStream>();
        quote!(&[$markers])
    }

    fn attrs(&self, attrs: Attrs, tail: &mut Tail) -> TokenStream {
        let mut markers = Vec::new();
        for Attribute { name, attr_type } in desugar_attrs(attrs) {
            markers.push(match attr_type {
                AttrType::Normal { value } => {
                    tail.push_str(" ");
                    let name_marker = self.name(name, tail);
                    tail.push_str("=\"");
                    let value_marker = self.markup(value, tail);
                    tail.push_str("\"");
                    quote!(maud::marker::attribute($name_marker, { $value_marker });)
                },
                AttrType::Empty { toggler: None } => {
                    tail.push_str(" ");
                    let name_marker = self.name(name, tail);
                    quote!(maud::marker::attribute($name_marker, ());)
                },
                AttrType::Empty { toggler: Some(toggler) } => {
                    let head = desugar_toggler(toggler);
                    tail.cut_once(move |mut tail| {
                        tail.push_str(" ");
                        let name_marker = self.name(name, &mut tail);
                        let attr_marker = quote!(maud::marker::attribute($name_marker, ()););
                        let body = tail.finish(attr_marker);
                        quote!($head { $body })
                    })
                },
            });
        }
        markers.into_iter().collect()
    }

    fn special(&self, Special { head, body }: Special, tail: Tail) -> TokenStream {
        let body = self.block(body, tail);
        quote!($head $body)
    }
}

////////////////////////////////////////////////////////

fn desugar_attrs(Attrs { classes_static, classes_toggled, ids, attrs }: Attrs) -> Vec<Attribute> {
    let classes = desugar_classes_or_ids("class", classes_static, classes_toggled);
    let ids = desugar_classes_or_ids("id", ids, vec![]);
    classes.into_iter().chain(ids).chain(attrs).collect()
}

fn desugar_classes_or_ids(
    attr_name: &'static str,
    values_static: Vec<ClassOrId>,
    values_toggled: Vec<(ClassOrId, Toggler)>,
) -> Option<Attribute> {
    if values_static.is_empty() && values_toggled.is_empty() {
        return None;
    }
    let mut markups = Vec::new();
    let mut leading_space = false;
    for symbol in values_static {
        markups.extend(prepend_leading_space(symbol, &mut leading_space));
    }
    for (symbol, toggler) in values_toggled {
        let body = Block {
            markups: prepend_leading_space(symbol, &mut leading_space),
            span: toggler.cond_span,
        };
        let head = desugar_toggler(toggler);
        markups.push(Markup::Special(Special { head, body }));
    }
    Some(Attribute {
        name: TokenStream::from(TokenTree {
            kind: TokenNode::Term(Term::intern(attr_name)),
            span: Span::def_site(),  // TODO
        }),
        attr_type: AttrType::Normal {
            value: Markup::Block(Block {
                markups,
                span: Span::def_site(),  // TODO
            }),
        },
    })
}

fn prepend_leading_space(symbol: TokenStream, leading_space: &mut bool) -> Vec<Markup> {
    let mut markups = Vec::new();
    if *leading_space {
        markups.push(Markup::Literal {
            content: " ".to_owned(),
            span: span_tokens(symbol.clone()),
        });
    }
    *leading_space = true;
    markups.push(Markup::Symbol { symbol });
    markups
}

fn desugar_toggler(Toggler { mut cond, cond_span }: Toggler) -> TokenStream {
    // If the expression contains an opening brace `{`,
    // wrap it in parentheses to avoid parse errors
    if cond.clone().into_iter().any(|token| match token.kind {
        TokenNode::Group(Delimiter::Brace, _) => true,
        _ => false,
    }) {
        cond = TokenStream::from(TokenTree {
            kind: TokenNode::Group(Delimiter::Parenthesis, cond),
            span: cond_span,
        });
    }
    quote!(if $cond)
}

fn span_tokens<I: IntoIterator<Item=TokenTree>>(tokens: I) -> Span {
    tokens
        .into_iter()
        .fold(None, |span: Option<Span>, token| Some(match span {
            None => token.span,
            Some(span) => span.join(token.span).unwrap_or(span),
        }))
        .unwrap_or(Span::def_site())
}

////////////////////////////////////////////////////////

struct Tail {
    output_ident: TokenTree,
    tail: String,
}

impl Tail {
    fn new(output_ident: TokenTree) -> Tail {
        Tail {
            output_ident,
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

    fn _cut(&mut self) -> TokenStream {
        if self.tail.is_empty() {
            return TokenStream::empty();
        }
        let push_str_expr = {
            let output_ident = self.output_ident.clone();
            let string = TokenNode::Literal(Literal::string(&self.tail));
            quote!($output_ident.push_str($string);)
        };
        self.tail.clear();
        push_str_expr
    }

    fn cut_once<F>(&mut self, callback: F) -> TokenStream where
        F: FnOnce(Tail) -> TokenStream,
    {
        self.cut_many(move |kitsune| callback(kitsune.fork()))
    }

    fn cut_many<F>(&mut self, callback: F) -> TokenStream where
        F: FnOnce(Kitsune) -> TokenStream,
    {
        let push_str_expr = self._cut();
        let next_expr = callback(Kitsune::new(self.output_ident.clone()));
        quote!($push_str_expr $next_expr)
    }

    fn finish(mut self, main_expr: TokenStream) -> TokenStream {
        let push_str_expr = self._cut();
        quote!($main_expr $push_str_expr)
    }
}

struct Kitsune {
    output_ident: TokenTree,
}

impl Kitsune {
    fn new(output_ident: TokenTree) -> Kitsune {
        Kitsune { output_ident }
    }

    fn fork(&self) -> Tail {
        Tail::new(self.output_ident.clone())
    }
}
