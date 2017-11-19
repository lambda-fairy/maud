use rustc::lint::{LateContext, LintArray, LintContext, LintPass};
use syntax::codemap::Span;

use maud_lint::MaudLintPass;

declare_lint! {
    pub MAUD_FROB,
    Warn,
    "frob!"
}

pub struct Frob;

impl LintPass for Frob {
    fn get_lints(&self) -> LintArray {
        lint_array![MAUD_FROB]
    }
}

impl<'a, 'tcx> MaudLintPass<'a, 'tcx> for Frob {
    fn check_literal(&mut self, cx: &LateContext<'a, 'tcx>, literal: String, span: Span) {
        cx.struct_span_lint(MAUD_FROB, span, &literal).emit();
    }
}
