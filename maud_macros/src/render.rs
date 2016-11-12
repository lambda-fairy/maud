use syntax::ast::{Expr, Ident, Pat, Stmt};
use syntax::ext::base::ExtCtxt;
use syntax::parse::token;
use syntax::ptr::P;
use syntax::tokenstream::TokenTree;

use maud::Escaper;

pub struct Renderer<'cx, 'a: 'cx> {
    pub cx: &'cx ExtCtxt<'a>,
    writer: Ident,
    stmts: Vec<Stmt>,
    tail: String,
}

impl<'cx, 'a> Renderer<'cx, 'a> {
    /// Creates a new `Renderer` using the given extension context.
    pub fn new(cx: &'cx ExtCtxt<'a>) -> Renderer<'cx, 'a> {
        let writer = token::gensym_ident("__maud_writer");
        Renderer {
            cx: cx,
            writer: writer,
            stmts: vec![],
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
                quote_expr!(self.cx, $w.push_str($s))
            };
            let stmt = self.wrap_stmt(expr);
            self.stmts.push(stmt);
            self.tail.clear();
        }
    }

    /// Reifies the `Renderer` into a block of markup.
    pub fn into_expr(mut self, size_hint: usize) -> P<Expr> {
        let Renderer { cx, writer, stmts, .. } = { self.flush(); self };
        quote_expr!(cx, {
            let mut $writer = ::std::string::String::with_capacity($size_hint);
            $stmts
            ::maud::PreEscaped($writer)
        })
    }

    /// Reifies the `Renderer` into a raw list of statements.
    pub fn into_stmts(mut self) -> Vec<Stmt> {
        let Renderer { stmts, .. } = { self.flush(); self };
        stmts
    }

    /// Pushes a statement, flushing the tail buffer in the process.
    fn push(&mut self, stmt: Stmt) {
        self.flush();
        self.stmts.push(stmt);
    }

    /// Pushes a literal string to the tail buffer.
    fn push_str(&mut self, s: &str) {
        self.tail.push_str(s);
    }

    /// Ignores the result of an expression.
    fn wrap_stmt(&self, expr: P<Expr>) -> Stmt {
        quote_stmt!(self.cx, $expr).unwrap()
    }

    /// Appends a literal string.
    pub fn string(&mut self, s: &str) {
        self.push_str(&html_escape(s));
    }

    /// Appends the result of an expression.
    pub fn splice(&mut self, expr: P<Expr>) {
        let w = self.writer;
        let expr = quote_expr!(self.cx, { use ::maud::RenderOnce; $expr.render_once_to(&mut $w) });
        let stmt = self.wrap_stmt(expr);
        self.push(stmt);
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
    pub fn emit_if(&mut self, if_cond: Vec<TokenTree>, if_body: Vec<Stmt>,
                   else_body: Option<Vec<Stmt>>) {
        let stmt = match else_body {
            None => quote_stmt!(self.cx, if $if_cond { $if_body }),
            Some(else_body) =>
                quote_stmt!(self.cx, if $if_cond { $if_body } else { $else_body }),
        }.unwrap();
        self.push(stmt);
    }

    /// Emits an `while` expression.
    ///
    /// The condition is a token tree (not an expression) so we don't
    /// need to special-case `while let`.
    pub fn emit_while(&mut self, cond: Vec<TokenTree>, body: Vec<Stmt>) {
        let stmt = quote_stmt!(self.cx, while $cond { $body }).unwrap();
        self.push(stmt);
    }

    pub fn emit_for(&mut self, pattern: P<Pat>, iterable: P<Expr>, body: Vec<Stmt>) {
        let stmt = quote_stmt!(self.cx, for $pattern in $iterable { $body }).unwrap();
        self.push(stmt);
    }

    pub fn emit_match(&mut self, match_var: P<Expr>, match_body: Vec<TokenTree>) {
        let stmt = quote_stmt!(self.cx, match $match_var { $match_body }).unwrap();
        self.push(stmt);
    }

    pub fn emit_let(&mut self, pattern: P<Pat>, rhs: P<Expr>, body: Vec<Stmt>) {
        let stmt = quote_stmt!(self.cx, { let $pattern = $rhs; $body }).unwrap();
        self.push(stmt);
    }
}

fn html_escape(s: &str) -> String {
    use std::fmt::Write;
    let mut buffer = String::new();
    Escaper::new(&mut buffer).write_str(s).unwrap();
    buffer
}
