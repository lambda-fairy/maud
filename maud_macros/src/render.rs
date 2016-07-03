use syntax::ast::{Expr, Ident, Pat, Stmt};
use syntax::codemap::DUMMY_SP;
use syntax::ext::base::ExtCtxt;
use syntax::parse::token;
use syntax::ptr::P;
use syntax::tokenstream::TokenTree;

use maud::Escaper;

pub struct Renderer<'cx, 'a: 'cx> {
    pub cx: &'cx ExtCtxt<'a>,
    writer: Ident,
    result: Ident,
    loop_label: Vec<TokenTree>,
    stmts: Vec<Stmt>,
    tail: String,
}

impl<'cx, 'a> Renderer<'cx, 'a> {
    /// Creates a new `Renderer` using the given extension context.
    pub fn new(cx: &'cx ExtCtxt<'a>) -> Renderer<'cx, 'a> {
        let writer = token::gensym_ident("__maud_writer");
        let result = token::gensym_ident("__maud_result");
        // Silence "duplicate loop labels" warning by appending ExpnId to label
        // FIXME This is a gross hack and should be replaced ASAP
        // See issues #36 and #37
        let loop_label = token::gensym_ident(&format!("__maud_loop_label_{}", cx.backtrace.into_u32()));
        Renderer {
            cx: cx,
            writer: writer,
            result: result,
            loop_label: vec![TokenTree::Token(DUMMY_SP, token::Lifetime(loop_label))],
            stmts: vec![],
            tail: String::new(),
        }
    }

    /// Creates a new `Renderer` under the same context as `self`.
    pub fn fork(&self) -> Renderer<'cx, 'a> {
        Renderer {
            cx: self.cx,
            writer: self.writer,
            result: self.result,
            loop_label: self.loop_label.clone(),
            stmts: Vec::new(),
            tail: String::new(),
        }
    }

    /// Flushes the tail buffer, emitting a single `.write_str()` call.
    fn flush(&mut self) {
        if !self.tail.is_empty() {
            let expr = {
                let w = self.writer;
                let s = &*self.tail;
                quote_expr!(self.cx, $w.write_str($s))
            };
            let stmt = self.wrap_try(expr);
            self.stmts.push(stmt);
            self.tail.clear();
        }
    }

    /// Reifies the `Renderer` into a block of markup.
    pub fn into_expr(mut self, writer_expr: Vec<TokenTree>) -> P<Expr> {
        let Renderer { cx, writer, result, loop_label, stmts, .. } = { self.flush(); self };
        quote_expr!(cx, {
            let mut $result = Ok(());
            $loop_label: loop {
                #[allow(unused_imports)]
                use ::std::fmt::Write;
                match &mut $writer_expr {
                    $writer => {
                        $writer as &mut ::std::fmt::Write;
                        $stmts
                    }
                }
                break $loop_label;
            }
            $result
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

    /// Wraps an expression in a `try!` call.
    fn wrap_try(&self, expr: P<Expr>) -> Stmt {
        let result = self.result;
        let loop_label = &self.loop_label;
        quote_stmt!(
            self.cx,
            match $expr {
                Ok(()) => {},
                Err(e) => {
                    $result = Err(e);
                    break $loop_label;
                }
            }).unwrap()
    }

    /// Appends a literal string.
    pub fn string(&mut self, s: &str) {
        self.push_str(&html_escape(s));
    }

    /// Appends the result of an expression, with the specified escaping method.
    pub fn splice(&mut self, expr: P<Expr>) {
        let w = self.writer;
        let expr = quote_expr!(self.cx, { use ::maud::RenderOnce; $expr.render_once(&mut *$w) });
        let stmt = self.wrap_try(expr);
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

    pub fn emit_for(&mut self, pattern: P<Pat>, iterable: P<Expr>, body: Vec<Stmt>) {
        let stmt = quote_stmt!(self.cx, for $pattern in $iterable { $body }).unwrap();
        self.push(stmt);
    }

    pub fn emit_match(&mut self, match_var: P<Expr>, match_body: Vec<TokenTree>) {
        let stmt = quote_stmt!(self.cx, match $match_var { $match_body }).unwrap();
        self.push(stmt);
    }

    pub fn emit_call(&mut self, func: P<Expr>) {
        let w = self.writer;
        let expr = quote_expr!(self.cx, ($func)(&mut *$w));
        let stmt = self.wrap_try(expr);
        self.push(stmt);
    }
}

fn html_escape(s: &str) -> String {
    use std::fmt::Write;
    let mut escaper = Escaper::new(String::new());
    escaper.write_str(s).unwrap();
    escaper.into_inner()
}
