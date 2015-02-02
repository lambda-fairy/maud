use std::borrow::IntoCow;
use syntax::ast::{Expr, ExprParen, Ident, Stmt};
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
    pub fn new(cx: &'cx ExtCtxt<'s>) -> Renderer<'cx, 's> {
        Renderer {
            cx: cx,
            stmts: vec![],
            w: Ident::new(token::intern("w")),
        }
    }

    pub fn into_expr(self) -> P<Expr> {
        let Renderer { cx, stmts, w } = self;
        quote_expr!(cx,
            ::maud::rt::make_markup(|&: $w: &mut ::std::fmt::Writer| -> Result<(), ::std::fmt::Error> {
                $stmts
                Ok(())
            }))
    }

    fn make_stmts<F>(&self, f: F) -> Vec<P<Stmt>> where
        F: FnOnce(&mut Renderer<'cx, 's>)
    {
        let mut render = Renderer {
            cx: self.cx,
            stmts: vec![],
            w: self.w,
        };
        f(&mut render);
        render.stmts
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

    pub fn attribute_empty_if(&mut self, name: &str, expr: P<Expr>) {
        // Silence "unnecessary parentheses" warnings
        let expr = match expr.node {
            ExprParen(ref inner) => inner.clone(),
            _ => expr.clone(),
        };
        let stmts = self.make_stmts(|r| {
            r.write(" ");
            r.write(name);
        });
        let stmt = quote_stmt!(self.cx, if $expr { $stmts });
        self.stmts.push(stmt);
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
}
