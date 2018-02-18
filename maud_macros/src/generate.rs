use maud_htmlescape::Escaper;
use proc_macro::{Delimiter, Literal, quote, Spacing, Span, TokenNode, TokenStream, TokenTree};

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
                    self.block(Block { markups, span }, tail)
                } else {
                    self.markups(markups, tail)
                }
            },
            Markup::Literal { content, span } => self.literal(&content, span, tail),
            Markup::Splice { expr } => self.splice(expr, tail),
            Markup::Element { name, attrs, body } => self.element(name, attrs, body, tail),
            Markup::Let { tokens } => tail.cut_then(|_| tokens),
            Markup::If { segments } => {
                // TODO moelarry
                segments.into_iter().map(|segment| self.special(segment, tail)).collect()
            },
            Markup::Special(special) => self.special(special, tail),
            Markup::Match { .. } => TokenStream::empty(),  // TODO
        }
    }

    fn block(&self, Block { markups, span }: Block, tail: &mut Tail) -> TokenStream {
        tail.cut_then(move |tail| {
            let markups = self.markups(markups, tail);
            TokenStream::from(TokenTree {
                kind: TokenNode::Group(Delimiter::Brace, tail.finish(markups)),
                span,
            })
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

    fn splice(&self, expr: TokenStream, tail: &mut Tail) -> TokenStream {
        let output_ident = self.output_ident.clone();
        tail.cut_then(move |_| quote!({
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
        // let mut markers = Vec::new();
        let Attrs { classes_static, classes_toggled, ids, attrs } = attrs;
        if !classes_static.is_empty() || !classes_toggled.is_empty() {
            // TODO
        }
        // TODO
        TokenStream::empty()
    }

    fn special(&self, Special { head, body }: Special, tail: &mut Tail) -> TokenStream {
        tail.cut_then(move |tail| {
            let body = self.block(body, tail);
            quote!($head $body)
        })
    }
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

    fn cut(&mut self) -> TokenStream {
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

    fn cut_then<F>(&mut self, callback: F) -> TokenStream where
        F: FnOnce(&mut Tail) -> TokenStream,
    {
        let push_str_expr = self.cut();
        let next_expr = callback(self);
        quote!($push_str_expr $next_expr)
    }

    fn finish(&mut self, main_expr: TokenStream) -> TokenStream {
        let push_str_expr = self.cut();
        quote!($main_expr $push_str_expr)
    }
}
