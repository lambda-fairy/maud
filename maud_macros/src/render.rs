use std::borrow::IntoCow;
use syntax::ast::{Expr, Ident, Stmt};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use syntax::ptr::P;

use maud;

#[derive(Copy, PartialEq, Show)]
pub enum Escape {
    PassThru,
    Escape,
}

pub struct Renderer<'cx, 's: 'cx, 'o> {
    pub cx: &'cx mut ExtCtxt<'s>,
    stmts: &'o mut Vec<P<Stmt>>,
    w: Ident,
}

impl<'cx, 's, 'o> Renderer<'cx, 's, 'o> {
    pub fn with<F>(cx: &'cx mut ExtCtxt<'s>, f: F) -> P<Expr> where
        F: for<'o_> FnOnce(&mut Renderer<'cx, 's, 'o_>)
    {
        let mut stmts = vec![];
        let w = Ident::new(token::intern("w"));
        let cx = {
            let mut render = Renderer {
                cx: cx,
                stmts: &mut stmts,
                w: w,
            };
            f(&mut render);
            render.cx
        };
        quote_expr!(cx,
            ::maud::rt::make_markup(|&: $w: &mut ::std::fmt::Writer| -> Result<(), ::std::fmt::Error> {
                $stmts
                Ok(())
            }))
    }

    /// Push an expression statement, also wrapping it with `try!`.
    fn push(&mut self, expr: P<Expr>) {
        let stmt = self.make_stmt(expr);
        self.stmts.push(stmt);
    }

    /// Create an expression statement, also wrapping it with `try!`.
    fn make_stmt(&mut self, expr: P<Expr>) -> P<Stmt> {
        self.cx.stmt_expr(self.cx.expr_try(expr.span, expr))
    }

    /// Append a literal pre-escaped string.
    fn write(&mut self, s: &str) {
        let w = self.w;
        let expr = quote_expr!(self.cx, $w.write_str($s));
        self.push(expr);
    }

    /// Append a literal string, with the specified escaping method.
    pub fn string(&mut self, s: &str, escape: Escape) {
        let s = match escape {
            Escape::PassThru => s.into_cow(),
            Escape::Escape => maud::escape(s).into_cow(),
        };
        self.write(s.as_slice());
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
        self.push(expr);
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
        let s: String = [" ", name].concat();
        let s = &s[];
        let w = self.w;
        let expr = quote_expr!(self.cx,
            if $expr {
                $w.write_str($s)
            } else {
                Ok(())
            });
        self.push(expr);
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
