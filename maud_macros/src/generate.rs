use proc_macro::{Span, TokenStream, TokenTree};

use ast::*;
use build::Builder;

pub fn generate(markups: Vec<Markup>, output_ident: TokenTree) -> TokenStream {
    let generator = Generator::new(output_ident);
    let mut builder = generator.builder();
    generator.markups(markups, &mut builder);
    builder.build()
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

    fn markups(&self, markups: Vec<Markup>, builder: &mut Builder) {
        for markup in markups {
            self.markup(markup, builder);
        }
    }

    fn markup(&self, markup: Markup, builder: &mut Builder) {
        match markup {
            Markup::Block(Block { markups, span }) =>
                if markups.iter().any(|markup| matches!(*markup, Markup::Let { .. })) {
                    self.block(Block { markups, span }, builder);
                } else {
                    self.markups(markups, builder);
                },
            Markup::Literal { content, span } => builder.string(&content, span),
            Markup::Splice { expr } => builder.splice(expr),
            Markup::Element { name, attrs, body } => {
                builder.element_open_start(name.clone());
                self.attrs(attrs, builder);
                builder.element_open_end();  // TODO use a different marker for self-closing tags
                if let Some(body) = body {
                    self.markup(*body, builder);
                    builder.element_close(name);
                }
            },
            Markup::Let { tokens } => {
                builder.push(tokens);
            },
            Markup::If { segments } => {
                for segment in segments {
                    self.special(segment, builder);
                }
            },
            Markup::Special(special) => {
                self.special(special, builder);
            },
            Markup::Match { head, arms, arms_span } => {
                builder.push(head);
                let arms = {
                    let mut builder = self.builder();
                    for arm in arms {
                        self.special(arm, &mut builder);
                    }
                    builder.build()
                };
                builder.push_block(arms, arms_span);
            },
        }
    }

    fn block(&self, Block { markups, span }: Block, builder: &mut Builder) {
        let stmts = {
            let mut builder = self.builder();
            self.markups(markups, &mut builder);
            builder.build()
        };
        builder.push_block(stmts, span);
    }

    fn attrs(&self, attrs: Attrs, builder: &mut Builder) {
        let Attrs { classes_static, classes_toggled, ids, attrs } = attrs;
        if !classes_static.is_empty() || !classes_toggled.is_empty() {
            builder.attribute_start_str("class", Span::def_site());  // TODO span
            let mut leading_space = false;
            for name in classes_static {
                builder.class_or_id(name, leading_space);
                leading_space = true;
            }
            for (name, Toggler { cond, cond_span }) in classes_toggled {
                let body = {
                    let mut builder = self.builder();
                    builder.class_or_id(name, leading_space);
                    leading_space = true;
                    builder.build()
                };
                builder.if_expr(cond, cond_span, body);
            }
            builder.attribute_end();
        }
        if !ids.is_empty() {
            builder.attribute_start_str("id", Span::def_site());  // TODO span
            for (i, name) in ids.into_iter().enumerate() {
                builder.class_or_id(name, i > 0);
            }
            builder.attribute_end();
        }
        for Attribute { name, attr_type } in attrs {
            match attr_type {
                AttrType::Normal { value } => {
                    builder.attribute_start(name);
                    self.markup(value, builder);
                    builder.attribute_end();
                },
                AttrType::Empty { toggler: None } => {
                    builder.attribute_empty(name);
                },
                AttrType::Empty { toggler: Some(Toggler { cond, cond_span }) } => {
                    let body = {
                        let mut builder = self.builder();
                        builder.attribute_empty(name);
                        builder.build()
                    };
                    builder.if_expr(cond, cond_span, body);
                },
            }
        }
    }

    fn special(&self, Special { head, body }: Special, builder: &mut Builder) {
        builder.push(head);
        self.block(body, builder);
    }
}
