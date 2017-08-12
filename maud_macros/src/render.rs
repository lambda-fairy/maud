use proc_macro::{Literal, TokenNode, TokenStream, TokenTree};
use proc_macro::quote;

use maud_htmlescape::Escaper;

pub struct Renderer {
    output_ident: TokenTree,
    stmts: Vec<TokenStream>,
    tail: String,
}

impl Renderer {
    /// Creates a new `Renderer`.
    pub fn new(output_ident: TokenTree) -> Renderer {
        Renderer {
            output_ident,
            stmts: Vec::new(),
            tail: String::new(),
        }
    }

    /// Flushes the tail buffer, emitting a single `.push_str()` call.
    fn flush(&mut self) {
        if !self.tail.is_empty() {
            let expr = {
                let output_ident = self.output_ident.clone();
                let string = TokenNode::Literal(Literal::string(&self.tail));
                quote!($output_ident.push_str($string);)
            };
            self.stmts.push(expr);
            self.tail.clear();
        }
    }

    /// Reifies the `Renderer` into a raw list of statements.
    pub fn build(mut self) -> TokenStream {
        let Renderer { stmts, .. } = { self.flush(); self };
        stmts.into_iter().collect()
    }

    /// Pushes a statement, flushing the tail buffer in the process.
    pub fn push<T>(&mut self, stmt: T) where T: Into<TokenStream> {
        self.flush();
        self.stmts.push(stmt.into())
    }

    /// Pushes a literal string to the tail buffer.
    fn push_str(&mut self, s: &str) {
        self.tail.push_str(s);
    }

    /// Appends a literal string.
    pub fn string(&mut self, s: &str) {
        self.push_str(&html_escape(s));
    }

    /// Appends the result of an expression.
    pub fn splice(&mut self, expr: TokenStream) {
        let output_ident = self.output_ident.clone();
        self.push(quote!({
            extern crate maud;
            // Create a local trait alias so that autoref works
            trait Render: maud::Render {
                fn __maud_render_to(&self, output_ident: &mut String) {
                    maud::Render::render_to(self, output_ident);
                }
            }
            impl<T: maud::Render> Render for T {}
            $expr.__maud_render_to(&mut $output_ident);
        }));
    }

    pub fn element_open_start(&mut self, name: &str) {
        self.push_str("<");
        self.push_str(name);
    }

    pub fn attribute_start(&mut self, name: &str) {
        self.push_str(" ");
        self.push_str(name);
        self.push_str("=\"");
    }

    pub fn attribute_empty(&mut self, name: &str) {
        self.push_str(" ");
        self.push_str(name);
    }

    pub fn attribute_end(&mut self) {
        self.push_str("\"");
    }

    pub fn element_open_end(&mut self) {
        self.push_str(">");
    }

    pub fn element_close(&mut self, name: &str) {
        self.push_str("</");
        self.push_str(name);
        self.push_str(">");
    }

    /// Emits an `if` expression.
    ///
    /// The condition is a token tree (not an expression) so we don't
    /// need to special-case `if let`.
    pub fn emit_if(&mut self, if_cond: TokenStream, if_body: TokenStream,
                   else_body: Option<TokenStream>) {
        let stmt = match else_body {
            None => quote!(if $if_cond { $if_body }),
            Some(else_body) => quote!(if $if_cond { $if_body } else { $else_body }),
        };
        self.push(stmt);
    }
}

fn html_escape(s: &str) -> String {
    use std::fmt::Write;
    let mut buffer = String::new();
    Escaper::new(&mut buffer).write_str(s).unwrap();
    buffer
}
