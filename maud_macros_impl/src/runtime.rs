use std::collections::HashMap;

use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::quote;

use crate::expand;
use crate::generate::desugar_attrs;
use crate::{ast::*, escape, expand_from_parsed, expand_runtime_from_parsed};

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
                for MatchArm { head, body } in arms {
                    tt.extend(head.clone());
                    tt.extend(self.get_block(&format!("{}{{", &head.to_string()), body));
                }

                let mut body = TokenTree::Group(Group::new(Delimiter::Brace, tt));
                body.set_span(arms_span.collapse());
                build.push_format_arg(quote!(#head #body));
            }
        }
    }

    fn block(&self, block: Block, build: &mut RuntimeBuilder) {
        build.push_format_arg(self.get_block("    {", block));
    }

    fn get_block(&self, scan_head: &str, block: Block) -> TokenStream {
        if let Some(raw_body) = block.raw_body {
            expand_runtime_from_parsed(raw_body, block.markups, &scan_head)
        } else {
            expand_from_parsed(block.markups, 0)
        }
    }

    fn special(&self, segments: Vec<Special>, build: &mut RuntimeBuilder) {
        let output_ident =
            TokenTree::Ident(Ident::new("__maud_special_output", Span::mixed_site()));
        let mut tt = TokenStream::new();
        for Special { head, body, .. } in segments {
            let body = self.get_block(&format!("{}{{", head.to_string()), body);
            tt.extend(quote! {
                #head {
                    ::maud::Render::render_to(&#body, &mut #output_ident);
                }
            });
        }
        build.push_format_arg(quote! {{
            extern crate maud;
            let mut #output_ident = ::maud::macro_private::String::new();
            #tt
            ::maud::PreEscaped(#output_ident)
        }});
    }

    fn splice(&self, expr: TokenStream, build: &mut RuntimeBuilder) {
        build.push_format_arg(expr);
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

                    build.push_format_arg(quote! {
                        if let Some(#inner_value) = (#cond) {
                            #body
                        } else {
                            ::maud::PreEscaped("".to_owned())
                        }
                    });
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

                    build.push_format_arg(quote! {
                        if (#cond) {
                            #body
                        } else {
                            ::maud::PreEscaped("".to_owned())
                        }
                    });
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
        escape::escape_to_string(&string, &mut s);
        self.push_str(&s);
    }

    fn push_format_arg(&mut self, expr: TokenStream) {
        let arg_track = self.arg_track.to_string();

        if let Some(ref vars) = self.vars_ident {
            self.tokens.extend(quote! {
                #vars.insert(#arg_track, {
                    extern crate maud;
                    let mut buf = ::maud::macro_private::String::new();
                    ::maud::macro_private::render_to!(&(#expr), &mut buf);
                    buf
                });
            });
        }

        self.arg_track = self.arg_track + 1;
        self.commands.push(Command::Variable(arg_track.to_string()));
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
    Variable(String),
    VariableFromVariable(String),
}

pub struct Interpreter {
    pub commands: Vec<Command>,
}

impl Interpreter {
    pub fn run(&self, variables: &HashMap<&str, String>) -> Result<String, String> {
        let mut rv = String::new();
        for command in &self.commands {
            match command {
                Command::String(s) => rv.push_str(s),
                Command::Variable(v) => {
                    let s = variables
                        .get(v.as_str())
                        .ok_or_else(|| format!("unknown var: {:?}", v))?;
                    rv.push_str(&s);
                }
                Command::VariableFromVariable(v) => {
                    let v = variables
                        .get(v.as_str())
                        .ok_or_else(|| format!("unknown var: {:?}", v))?;
                    let s = variables
                        .get(v.as_str())
                        .ok_or_else(|| format!("unknown secondary var: {:?}", v))?;
                    rv.push_str(&s);
                }
            }
        }

        Ok(rv)
    }
}
