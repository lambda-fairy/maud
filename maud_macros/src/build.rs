use proc_macro::{Delimiter, Literal, Spacing, Span, Term, TokenNode, TokenStream, TokenTree};
use proc_macro::quote;

use maud_htmlescape::Escaper;

pub struct Builder {
    output_ident: TokenTree,
    stmts: Vec<TokenStream>,
    tail: String,
}

impl Builder {
    /// Creates a new `Builder`.
    pub fn new(output_ident: TokenTree) -> Builder {
        Builder {
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

    /// Reifies the `Builder` into a raw list of statements.
    pub fn build(mut self) -> TokenStream {
        let Builder { stmts, .. } = { self.flush(); self };
        stmts.into_iter().collect()
    }

    /// Pushes a statement, flushing the tail buffer in the process.
    pub fn push<T>(&mut self, stmt: T) where T: Into<TokenStream> {
        self.flush();
        self.stmts.push(stmt.into())
    }

    fn push_marker(&mut self, stmt: TokenStream) {
        self.stmts.push(stmt);
    }

    /// Pushes a literal string to the tail buffer.
    fn push_str(&mut self, s: &str) {
        self.tail.push_str(s);
    }

    /// Appends a literal string.
    pub fn string(&mut self, s: &str, span: Span) {
        let marker = TokenTree {
            kind: TokenNode::Literal(Literal::string(s)),
            span,
        };
        self.push_str(&html_escape(s));
        self.push_marker(quote!(maud::marker::literal(&[$marker]);));
    }

    /// Appends a class or ID name, with an optional space before it.
    pub fn class_or_id(&mut self, name: TokenStream, leading_space: bool) {
        if leading_space {
            self.push_str(" ");
        }
        self.name_with_marker(name, quote!(maud::marker::literal));
    }

    fn name_with_marker(&mut self, name: TokenStream, marker_method: TokenStream) {
        let mut markers = Vec::new();
        for token in name {
            let s = token.to_string();
            markers.push(TokenTree {
                kind: TokenNode::Literal(Literal::string(&s)),
                span: token.span,
            });
            self.push_str(&html_escape(&s));
            markers.push(TokenTree {
                kind: TokenNode::Op(',', Spacing::Alone),
                span: token.span,
            });
        }
        let markers = markers.into_iter().collect::<TokenStream>();
        self.push_marker(quote!($marker_method(&[$markers]);));
    }

    /// Appends the result of an expression.
    pub fn splice(&mut self, expr: TokenStream) {
        let output_ident = self.output_ident.clone();
        self.push(quote!({
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

    pub fn element_open_start(&mut self, name: TokenStream) {
        self.push_str("<");
        self.name_with_marker(name, quote!(maud::marker::element_open_start));
    }

    pub fn attribute_start(&mut self, name: TokenStream) {
        self.push_str(" ");
        self.name_with_marker(name, quote!(maud::marker::attribute_start));
        self.push_str("=\"");
    }

    pub fn attribute_start_str(&mut self, name: &str, span: Span) {
        let name = TokenTree {
            kind: TokenNode::Term(Term::intern(name)),
            span,
        };
        self.attribute_start(TokenStream::from(name));
    }

    pub fn attribute_empty(&mut self, name: TokenStream) {
        self.push_str(" ");
        self.name_with_marker(name, quote!(maud::marker::attribute_empty));
    }

    pub fn attribute_end(&mut self) {
        self.push_str("\"");
        self.push_marker(quote!(maud::marker::attribute_end();));
    }

    pub fn element_open_end(&mut self) {
        self.push_str(">");
        self.push_marker(quote!(maud::marker::element_open_end();));
    }

    pub fn element_close(&mut self, name: TokenStream) {
        let name = name.into_iter().map(|token| token.to_string()).collect::<String>();
        self.push_str("</");
        self.push_str(&name);
        self.push_str(">");
        self.push_marker(quote!(maud::marker::element_close();));
    }

    /// Emits an `if` expression.
    ///
    /// The condition is a token stream (not an expression) so we don't
    /// need to special-case `if let`.
    pub fn emit_if(
        &mut self,
        mut cond: TokenStream,
        cond_span: Span,
        body: TokenStream,
    ) {
        // If the condition contains an opening brace `{`,
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
        self.push(quote!(if $cond { $body }));
    }
}

fn html_escape(s: &str) -> String {
    use std::fmt::Write;
    let mut buffer = String::new();
    Escaper::new(&mut buffer).write_str(s).unwrap();
    buffer
}
