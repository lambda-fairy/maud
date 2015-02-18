use std::borrow::IntoCow;
use syntax::ast::{Expr, Ident, Stmt};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use syntax::ptr::P;

use maud;

#[derive(Copy)]
pub enum Escape {
    PassThru,
    Escape,
}

pub struct Renderer<'cx, 's: 'cx> {
    pub cx: &'cx ExtCtxt<'s>,
    stmts: Vec<P<Stmt>>,
    w: Ident,
}

impl<'cx, 's> Renderer<'cx, 's> {
    /// Create a new `Renderer` using the given extension context.
    pub fn new(cx: &'cx ExtCtxt<'s>) -> Renderer<'cx, 's> {
        Renderer {
            cx: cx,
            stmts: vec![],
            w: Ident::new(token::intern("w")),
        }
    }

    /// Create a new `Renderer` under the same context as `self`.
    pub fn fork(&self) -> Renderer<'cx, 's> {
        Renderer {
            cx: self.cx,
            stmts: vec![],
            w: self.w,
        }
    }

    /// Reify the `Renderer` into a block of markup.
    pub fn into_expr(self) -> P<Expr> {
        let Renderer { cx, stmts, w } = self;
        quote_expr!(cx,
            ::maud::rt::make_markup(|$w: &mut ::std::fmt::Write| -> Result<(), ::std::fmt::Error> {
                $stmts
                Ok(())
            }))
    }

    /// Reify the `Renderer` into a raw list of statements.
    pub fn into_stmts(self) -> Vec<P<Stmt>> {
        let Renderer { stmts, .. } = self;
        stmts
    }

    /// Append the list of statements to the output.
    pub fn push_stmts(&mut self, mut stmts: Vec<P<Stmt>>) {
        self.stmts.append(&mut stmts);
    }

    /// Push an expression statement, also wrapping it with `try!`.
    fn push_try(&mut self, expr: P<Expr>) {
        let stmt = self.cx.stmt_expr(self.cx.expr_try(expr.span, expr));
        self.stmts.push(stmt);
    }

    /// Append a literal pre-escaped string.
    fn write(&mut self, s: &str) {
        let w = self.w;
        let expr = quote_expr!(self.cx, $w.write_str($s));
        self.push_try(expr);
    }

    /// Append a literal string, with the specified escaping method.
    pub fn string(&mut self, s: &str, escape: Escape) {
        let s = match escape {
            Escape::PassThru => s.into_cow(),
            Escape::Escape => maud::escape(s).into_cow(),
        };
        self.write(&s);
    }

    /// Append the result of an expression, with the specified escaping method.
    pub fn splice(&mut self, expr: P<Expr>, escape: Escape) {
        let w = self.w;
        let expr = match escape {
            Escape::PassThru =>
                quote_expr!(self.cx, ::maud::rt::write_fmt($w, $expr)),
            Escape::Escape =>
                quote_expr!(self.cx,
                    ::maud::rt::write_fmt(
                        &mut ::maud::rt::Escaper { inner: $w },
                        $expr)),
        };
        self.push_try(expr);
    }

    pub fn element_open_start(&mut self, name: &str) {
        self.write("<");
        self.write(name);
    }

    pub fn attribute_start(&mut self, name: &str) {
        self.write(" ");
        self.write(name);
        self.write("=\"");
    }

    pub fn attribute_empty(&mut self, name: &str) {
        self.write(" ");
        self.write(name);
    }

    pub fn attribute_end(&mut self) {
        self.write("\"");
    }

    pub fn element_open_end(&mut self) {
        self.write(">");
    }

    pub fn element_close(&mut self, name: &str) {
        self.write("</");
        self.write(name);
        self.write(">");
    }

    pub fn emit_if(&mut self, cond: P<Expr>, if_body: Vec<P<Stmt>>,
                   else_body: Option<Vec<P<Stmt>>>) {
        let stmt = match else_body {
            None => quote_stmt!(self.cx, if $cond { $if_body }),
            Some(else_body) =>
                quote_stmt!(self.cx, if $cond { $if_body } else { $else_body }),
        };
        self.stmts.push(stmt);
    }
}
