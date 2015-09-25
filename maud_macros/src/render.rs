use std::fmt::Write;
use syntax::ast::{Expr, Ident, Pat, Stmt, TokenTree, TtToken};
use syntax::codemap::DUMMY_SP;
use syntax::ext::base::ExtCtxt;
use syntax::parse::token;
use syntax::ptr::P;

use maud::Escaper;

#[derive(Copy, Clone)]
pub enum Escape {
    PassThru,
    Escape,
}

pub struct Renderer<'cx> {
    pub cx: &'cx ExtCtxt<'cx>,
    writer: Ident,
    result: Ident,
    loop_label: Vec<TokenTree>,
    stmts: Vec<P<Stmt>>,
    tail: String,
}

impl<'cx> Renderer<'cx> {
    /// Creates a new `Renderer` using the given extension context.
    pub fn new(cx: &'cx ExtCtxt<'cx>) -> Renderer<'cx> {
        let writer = token::gensym_ident("__maud_writer");
        let result = token::gensym_ident("__maud_result");
        let loop_label = token::gensym_ident("__maud_loop_label");
        Renderer {
            cx: cx,
            writer: writer,
            result: result,
            loop_label: vec![TtToken(DUMMY_SP, token::Lifetime(loop_label))],
            stmts: vec![],
            tail: String::new(),
        }
    }

    /// Creates a new `Renderer` under the same context as `self`.
    pub fn fork(&self) -> Renderer<'cx> {
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
    pub fn into_stmts(mut self) -> Vec<P<Stmt>> {
        let Renderer { stmts, .. } = { self.flush(); self };
        stmts
    }

    /// Pushes a statement, flushing the tail buffer in the process.
    fn push(&mut self, stmt: P<Stmt>) {
        self.flush();
        self.stmts.push(stmt);
    }

    /// Pushes a literal string to the tail buffer.
    fn push_str(&mut self, s: &str) {
        self.tail.push_str(s);
    }

    /// Wraps an expression in a `try!` call.
    fn wrap_try(&self, expr: P<Expr>) -> P<Stmt> {
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

    /// Appends a literal string, with the specified escaping method.
    pub fn string(&mut self, s: &str, escape: Escape) {
        let escaped;
        let s = match escape {
            Escape::PassThru => s,
            Escape::Escape => { escaped = html_escape(s); &*escaped },
        };
        self.push_str(s);
    }

    /// Appends the result of an expression, with the specified escaping method.
    pub fn splice(&mut self, expr: P<Expr>, escape: Escape) {
        let w = self.writer;
        let expr = match escape {
            Escape::PassThru =>
                quote_expr!(self.cx, write!($w, "{}", $expr)),
            Escape::Escape =>
                quote_expr!(self.cx,
                    write!(
                        ::maud::Escaper::new(&mut *$w),
                        "{}",
                        $expr)),
        };
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
    pub fn emit_if(&mut self, if_cond: Vec<TokenTree>, if_body: Vec<P<Stmt>>,
                   else_body: Option<Vec<P<Stmt>>>) {
        let stmt = match else_body {
            None => quote_stmt!(self.cx, if $if_cond { $if_body }),
            Some(else_body) =>
                quote_stmt!(self.cx, if $if_cond { $if_body } else { $else_body }),
        }.unwrap();
        self.push(stmt);
    }

    pub fn emit_for(&mut self, pattern: P<Pat>, iterable: P<Expr>, body: Vec<P<Stmt>>) {
        let stmt = quote_stmt!(self.cx, for $pattern in $iterable { $body }).unwrap();
        self.push(stmt);
    }

    pub fn emit_call(&mut self, func: P<Expr>) {
        let w = self.writer;
        let expr = quote_expr!(self.cx, ($func)(&mut *$w));
        let stmt = self.wrap_try(expr);
        self.push(stmt);
    }

    pub fn emit_call_box(&mut self, func: P<Expr>) {
        let w = self.writer;
        let expr = quote_expr!(self.cx,
            ::std::boxed::FnBox::call_box(
                $func,
                (&mut *$w as &mut ::std::fmt::Write,)));
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
