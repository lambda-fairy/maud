use rustc::hir::{Expr, ExprCall, ExprLit, ExprPath};
use rustc::lint::{LateContext, LateLintPass, LintArray, LintContext, LintPass};
use std::ascii::AsciiExt;
use super::util::match_def_path;
use syntax::ast::LitKind;

declare_lint! {
    pub MAUD_DOCTYPE,
    Warn,
    "suggest using the maud::DOCTYPE constant"
}

pub struct Doctype;

impl LintPass for Doctype {
    fn get_lints(&self) -> LintArray {
        lint_array![MAUD_DOCTYPE]
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Doctype {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        if_chain! {
            // It's a function call...
            if let ExprCall(ref path_expr, ref args) = expr.node;
            // ... where the argument is a literal "<!doctype html>"
            if let Some(first_arg) = args.first();
            if let ExprLit(ref lit) = first_arg.node;
            if let LitKind::Str(s, _) = lit.node;
            if s.as_str().eq_ignore_ascii_case("<!doctype html>");
            // ... and the callee is `maud::PreEscaped`
            if let ExprPath(ref qpath) = path_expr.node;
            let def_id = cx.tables.qpath_def(qpath, path_expr.hir_id).def_id();
            if match_def_path(cx, def_id, &["maud", "PreEscaped"]);
            then {
                cx.struct_span_lint(MAUD_DOCTYPE, expr.span,
                                    "use `maud::DOCTYPE` instead").emit();
            }
        }
    }
}
