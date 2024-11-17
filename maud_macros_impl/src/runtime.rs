use std::collections::HashMap;

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::quote;

use crate::{
    ast::*, escape, expand, expand_from_parsed, expand_runtime_from_parsed, generate::desugar_attrs,
};

pub fn generate(vars_ident: Option<TokenTree>, markups: Vec<Markup>) -> TokenStream {
    let mut build = RuntimeBuilder::new(vars_ident.clone());
    RuntimeGenerator::new().markups(markups, &mut build);
    build.finish()
}

pub fn build_interpreter(markups: Vec<Markup>) -> Interpreter {
    let mut build = RuntimeBuilder::new(None);
    RuntimeGenerator::new().markups(markups, &mut build);
    build.interpreter()
}

struct RuntimeGenerator {}

impl RuntimeGenerator {
    fn new() -> RuntimeGenerator {
        RuntimeGenerator {}
    }

    fn markups(&self, markups: Vec<Markup>, build: &mut RuntimeBuilder) {
        for markup in markups {
            self.markup(markup, build);
        }
    }

    fn markup(&self, markup: Markup, build: &mut RuntimeBuilder) {
        match markup {
            Markup::ParseError { .. } => {}
            Markup::Block(Block {
                markups,
                outer_span,
                raw_body,
            }) => {
                if markups
                    .iter()
                    .any(|markup| matches!(*markup, Markup::Let { .. }))
                {
                    self.block(
                        Block {
                            markups,
                            outer_span,
                            raw_body,
                        },
                        build,
                    );
                } else {
                    self.markups(markups, build);
                }
            }
            Markup::Literal { content, .. } => build.push_escaped(&content),
            Markup::Symbol { symbol } => self.name(symbol, build),
            Markup::Splice { expr, .. } => self.splice(expr, build),
            Markup::Element { name, attrs, body } => self.element(name, attrs, body, build),
            Markup::Let { tokens, .. } => {
                // this is a bit dicey
                build.tokens.extend(tokens);
            }
            Markup::Special { segments, .. } => self.special(segments, build),
            Markup::Match {
                head,
                arms,
                arms_span,
                ..
            } => {
                let mut tt = TokenStream::new();
                let mut sources = Vec::new();
                for (i, MatchArm { head, body }) in arms.into_iter().enumerate() {
                    if let Some(ref template_source) = body.raw_body {
                        sources.push(Some(template_source.clone()));
                    } else {
                        sources.push(None);
                    }
                    tt.extend(head.clone());
                    let partial = self.get_block(body);
                    tt.extend(quote! {{
                        let __maud_match_partial = #partial;
                        Box::new(|sources| __maud_match_partial(vec![sources[#i].clone()]))
                    }});
                }

                let mut body = TokenTree::Group(Group::new(Delimiter::Brace, tt));
                body.set_span(arms_span.collapse());
                build.push_lazy_format_arg(
                    quote!(#head #body),
                    sources,
                    &format!("match_expr: {}", head),
                );
            }
        }
    }

    fn block(&self, block: Block, build: &mut RuntimeBuilder) {
        let source = block.raw_body.clone();
        build.push_lazy_format_arg(self.get_block(block), vec![source], "block");
    }

    fn get_block(&self, block: Block) -> TokenStream {
        if block.raw_body.is_some() {
            expand_runtime_from_parsed(block.markups)
        } else {
            // necessary to avoid bogus sources
            let static_result = expand_from_parsed(block.markups, 0);
            quote! {{
                let __maud_static_result = (#static_result);
                let partial: ::maud::macro_private::PartialTemplate = Box::new(|_| Ok(__maud_static_result.into_string()));
                partial
            }}
        }
    }

    fn special(&self, segments: Vec<Special>, build: &mut RuntimeBuilder) {
        let mut tt = TokenStream::new();
        let mut sources = Vec::new();
        let mut varname = String::from("special: ");
        for (i, Special { head, body, .. }) in segments.into_iter().enumerate() {
            if let Some(ref template_source) = body.raw_body {
                varname.push_str(&normalize_source_for_hashing(head.to_string()));
                varname.push('\n');
                sources.push(Some(template_source.clone()));
            } else {
                sources.push(None);
            }

            let block = self.get_block(body);
            tt.extend(quote! {
                #head {
                    __maud_special_res.push((#i, #block));
                }
            });
        }
        let output = quote! {{
            extern crate maud;
            let mut __maud_special_res = Vec::new();
            #tt
            Box::new(move |sources| {
                let mut maud_special_output = ::maud::macro_private::String::new();
                for (source_i, subpartial) in __maud_special_res {
                    let new_sources = ::maud::macro_private::Vec::from([sources[source_i].clone()]);
                    maud_special_output.push_str(&subpartial(new_sources)?);
                }

                Ok(maud_special_output)
            })
        }};

        build.push_lazy_format_arg(output, sources, &varname);
    }

    fn splice(&self, expr: TokenStream, build: &mut RuntimeBuilder) {
        build.push_format_arg(expr, vec![None], "splice");
    }

    fn element(
        &self,
        name: TokenStream,
        attrs: Vec<Attr>,
        body: ElementBody,
        build: &mut RuntimeBuilder,
    ) {
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
                AttrType::Optional {
                    toggler: Toggler { cond, .. },
                } => {
                    let inner_value = quote!(inner_value);
                    let name_tok = name_to_string(name);
                    let body = expand(quote! {
                        (::maud::PreEscaped(" "))
                        (#name_tok)
                        (::maud::PreEscaped("=\""))
                        (#inner_value)
                        (::maud::PreEscaped("\""))
                    });

                    build.push_format_arg(
                        quote! {
                            if let Some(#inner_value) = (#cond) {
                                #body
                            } else {
                                ::maud::PreEscaped("".to_owned())
                            }
                        },
                        vec![None],
                        "optional_attr",
                    );
                }
                AttrType::Empty { toggler: None } => {
                    build.push_str(" ");
                    self.name(name, build);
                }
                AttrType::Empty {
                    toggler: Some(Toggler { cond, .. }),
                } => {
                    let name_tok = name_to_string(name);
                    let body = expand(quote! {
                        " "
                        (#name_tok)
                    });

                    build.push_format_arg(
                        quote! {
                            if (#cond) {
                                #body
                            } else {
                                ::maud::PreEscaped("".to_owned())
                            }
                        },
                        vec![None],
                        "empty_attr",
                    );
                }
            }
        }
    }
}

////////////////////////////////////////////////////////

struct RuntimeBuilder {
    vars_ident: Option<TokenTree>,
    tokens: Vec<TokenTree>,
    commands: Vec<Command>,
    arg_track: u32,
}

impl RuntimeBuilder {
    fn new(vars_ident: Option<TokenTree>) -> RuntimeBuilder {
        RuntimeBuilder {
            vars_ident,
            tokens: Vec::new(),
            commands: Vec::new(),
            arg_track: 0,
        }
    }

    fn push_str(&mut self, string: &str) {
        self.commands.push(Command::String(string.to_owned()));
    }

    fn push_escaped(&mut self, string: &str) {
        let mut s = String::new();
        escape::escape_to_string(string, &mut s);
        self.push_str(&s);
    }

    fn push_format_arg(
        &mut self,
        expr: TokenStream,
        template_sources: TemplateSourceContext,
        named_variable: &str,
    ) {
        self.push_lazy_format_arg(
            quote! {{
                extern crate maud;
                let mut buf = ::maud::macro_private::String::new();
                ::maud::macro_private::render_to!(&(#expr), &mut buf);
                ::maud::macro_private::Box::new(move |_| Ok(buf))
            }},
            template_sources,
            named_variable,
        );
    }

    fn push_lazy_format_arg(
        &mut self,
        expr: TokenStream,
        template_sources: TemplateSourceContext,
        named_variable: &str,
    ) {
        let variable_name = format!("{}_{}", self.arg_track, named_variable);

        if let Some(ref vars) = self.vars_ident {
            self.tokens.extend(quote! {
                #vars.insert(#variable_name, #expr);
            });
        }

        self.commands.push(Command::Variable {
            name: variable_name,
            template_sources,
        });

        self.arg_track += 1;
    }

    fn interpreter(self) -> Interpreter {
        Interpreter {
            commands: self.commands,
        }
    }

    fn finish(self) -> TokenStream {
        self.tokens.into_iter().collect::<TokenStream>()
    }
}

// /////// INTERPRETER

pub enum Command {
    String(String),
    Variable {
        name: String,
        template_sources: TemplateSourceContext,
    },
}

pub struct Interpreter {
    pub commands: Vec<Command>,
}

impl Interpreter {
    pub fn run(self, mut variables: HashMap<&str, PartialTemplate>) -> Result<String, String> {
        let mut rv = String::new();
        for command in self.commands {
            match command {
                Command::String(s) => rv.push_str(&s),
                Command::Variable {
                    name,
                    template_sources,
                } => {
                    let s = variables.remove(name.as_str()).ok_or_else(|| {
                        format!(
                            "unknown var: {:?}\nremaining variables: {:?}",
                            name,
                            variables.keys()
                        )
                    })?;
                    rv.push_str(&s(template_sources)?);
                }
            }
        }

        Ok(rv)
    }
}

pub type TemplateSource = TokenStream;
pub type TemplateSourceContext = Vec<Option<TemplateSource>>;

// partial templates are generated code that take their own sourcecode for live reloading.
pub type PartialTemplate = Box<dyn FnOnce(TemplateSourceContext) -> Result<String, String>>;

// we add hashes of source code to our variable names to prevent the chances of mis-rendering
// something, such as when a user swaps blocks around in the template
fn normalize_source_for_hashing(mut input: String) -> String {
    input.retain(|c| !c.is_ascii_whitespace());

    input
}
