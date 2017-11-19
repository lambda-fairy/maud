use rustc::hir::{Expr, ExprAddrOf, ExprArray, ExprCall, ExprLit, ExprPath, MutImmutable};
use rustc::lint::{LateContext, LateLintPass, LintArray, LintPass};
use syntax::ast::{LitKind, StrStyle};
use syntax::codemap::Span;

use util::match_def_path;

#[allow(unused_variables)]
pub trait MaudLintPass<'a, 'tcx>: LintPass {
    fn check_literal(&mut self, cx: &LateContext<'a, 'tcx>, content: String, span: Span) {}
    fn check_element_open_start(&mut self, cx: &LateContext<'a, 'tcx>, name: String, span: Span) {}

    fn invalid_marker(&mut self, cx: &LateContext<'a, 'tcx>, expected: &'static str, span: Span) {}
}

pub struct UseMaudLintPass<L>(pub L);

impl<L: LintPass> LintPass for UseMaudLintPass<L> {
    fn get_lints(&self) -> LintArray {
        self.0.get_lints()
    }
}

impl<'a, 'tcx, L: MaudLintPass<'a, 'tcx>> LateLintPass<'a, 'tcx> for UseMaudLintPass<L> {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        if_chain! {
            if let ExprCall(ref path_expr, ref args) = expr.node;
            if let ExprPath(ref qpath) = path_expr.node;
            let def_id = cx.tables.qpath_def(qpath, path_expr.hir_id).def_id();
            then {
                if match_def_path(cx, def_id, &["maud", "marker", "literal"]) {
                    if let Some((content, span)) = args.first().and_then(extract_strings) {
                        self.0.check_literal(cx, content, span);
                    } else {
                        self.0.invalid_marker(cx, "literal", expr.span);
                    }
                } else if match_def_path(cx, def_id, &["maud", "marker", "element_open_start"]) {
                    if let Some((name, span)) = args.first().and_then(extract_strings) {
                        self.0.check_element_open_start(cx, name, span);
                    } else {
                        self.0.invalid_marker(cx, "element open start", expr.span);
                    }
                }
            }
        }
    }
}

fn extract_strings(expr: &Expr) -> Option<(String, Span)> {
    let args = if_chain! {
        if let ExprAddrOf(MutImmutable, ref expr) = expr.node;
        if let ExprArray(ref args) = expr.node;
        then {
            args
        } else {
            return None;
        }
    };
    let mut content = String::new();
    let mut span: Option<Span> = None;
    for expr in args {
        if_chain! {
            if let ExprLit(ref lit) = expr.node;
            if let LitKind::Str(s, StrStyle::Cooked) = lit.node;
            then {
                content.push_str(&s.as_str());
                if let Some(ref mut span) = span {
                    *span = span.to(lit.span);
                } else {
                    span = Some(lit.span);
                }
            } else {
                return None;
            }
        }
    }
    span.map(|span| (content, span))
}
