use syntax::ast::Ident;
use syntax::ext::base::ExtCtxt;
use syntax::symbol::Symbol;
use syntax::tokenstream::{TokenStream, TokenTree};

use maud::Escaper;

// FIXME(rust-lang/rust#40939):
// * Use `TokenStreamBuilder` instead of `Vec<TokenStream>`
// * Use `quote!()` instead of `quote_tokens!()`

pub struct Renderer<'cx, 'a: 'cx> {
    cx: &'cx ExtCtxt<'a>,
    writer: Ident,
    stmts: Vec<TokenStream>,
    tail: String,
}

impl<'cx, 'a> Renderer<'cx, 'a> {
    /// Creates a new `Renderer` using the given extension context.
    pub fn new(cx: &'cx ExtCtxt<'a>) -> Renderer<'cx, 'a> {
        let writer = Ident::with_empty_ctxt(Symbol::gensym("__maud_output"));
        Renderer {
            cx: cx,
            writer: writer,
            stmts: Vec::new(),
            tail: String::new(),
        }
    }

    /// Creates a new `Renderer` under the same context as `self`.
    pub fn fork(&self) -> Renderer<'cx, 'a> {
        Renderer {
            cx: self.cx,
            writer: self.writer,
            stmts: Vec::new(),
            tail: String::new(),
        }
    }

    /// Flushes the tail buffer, emitting a single `.push_str()` call.
    fn flush(&mut self) {
        if !self.tail.is_empty() {
            let expr = {
                let w = self.writer;
                let s = &*self.tail;
                quote_tokens!(self.cx, $w.push_str($s);)
            };
            self.stmts.push(expr.into_iter().collect());
            self.tail.clear();
        }
    }

    /// Reifies the `Renderer` into a block of markup.
    pub fn into_expr(mut self, size_hint: usize) -> TokenStream {
        let Renderer { cx, writer, stmts, .. } = { self.flush(); self };
        let stmts: Vec<TokenTree> = TokenStream::concat(stmts).into_trees().collect();
        quote_tokens!(cx, {
            let mut $writer = ::std::string::String::with_capacity($size_hint);
            $stmts
            ::maud::PreEscaped($writer)
        }).into_iter().collect()
    }

    /// Reifies the `Renderer` into a raw list of statements.
    pub fn into_stmts(mut self) -> TokenStream {
        let Renderer { stmts, .. } = { self.flush(); self };
        TokenStream::concat(stmts)
    }

    /// Pushes a statement, flushing the tail buffer in the process.
    fn push<T>(&mut self, stmt: T) where T: IntoIterator<Item=TokenTree> {
        self.flush();
        self.stmts.push(stmt.into_iter().collect())
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
        let w = self.writer;
        let expr: Vec<TokenTree> = expr.into_trees().collect();
        self.push(quote_tokens!(self.cx, {
            #[allow(unused_imports)]
            use ::maud::Render as __maud_Render;
            $expr.render_to(&mut $w);
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
        let if_cond: Vec<TokenTree> = if_cond.into_trees().collect();
        let if_body: Vec<TokenTree> = if_body.into_trees().collect();
        let stmt = match else_body {
            None => quote_tokens!(self.cx, if $if_cond { $if_body }),
            Some(else_body) => {
                let else_body: Vec<TokenTree> = else_body.into_trees().collect();
                quote_tokens!(self.cx, if $if_cond { $if_body } else { $else_body })
            },
        };
        self.push(stmt);
    }

    /// Emits an `while` expression.
    ///
    /// The condition is a token tree (not an expression) so we don't
    /// need to special-case `while let`.
    pub fn emit_while(&mut self, cond: TokenStream, body: TokenStream) {
        let cond: Vec<TokenTree> = cond.into_trees().collect();
        let body: Vec<TokenTree> = body.into_trees().collect();
        let stmt = quote_tokens!(self.cx, while $cond { $body });
        self.push(stmt);
    }

    pub fn emit_for(&mut self, pattern: TokenStream, iterable: TokenStream, body: TokenStream) {
        let pattern: Vec<TokenTree> = pattern.into_trees().collect();
        let iterable: Vec<TokenTree> = iterable.into_trees().collect();
        let body: Vec<TokenTree> = body.into_trees().collect();
        let stmt = quote_tokens!(self.cx, for $pattern in $iterable { $body });
        self.push(stmt);
    }

    pub fn emit_match(&mut self, match_var: TokenStream, match_body: TokenStream) {
        let match_var: Vec<TokenTree> = match_var.into_trees().collect();
        let match_body: Vec<TokenTree> = match_body.into_trees().collect();
        let stmt = quote_tokens!(self.cx, match $match_var { $match_body });
        self.push(stmt);
    }

    pub fn emit_let(&mut self, pattern: TokenStream, rhs: TokenStream, body: TokenStream) {
        let pattern: Vec<TokenTree> = pattern.into_trees().collect();
        let rhs: Vec<TokenTree> = rhs.into_trees().collect();
        let body: Vec<TokenTree> = body.into_trees().collect();
        let stmt = quote_tokens!(self.cx, { let $pattern = $rhs; $body });
        self.push(stmt);
    }
}

fn html_escape(s: &str) -> String {
    use std::fmt::Write;
    let mut buffer = String::new();
    Escaper::new(&mut buffer).write_str(s).unwrap();
    buffer
}
