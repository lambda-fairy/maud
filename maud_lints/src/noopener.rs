use rustc::hir::Expr;
use rustc::lint::{LateContext, LateLintPass, LintArray, LintContext, LintPass};
use syntax_pos::Span;

use util::*;

declare_lint! {
    pub MAUD_NOOPENER,
    Warn,
    r#"links with a `target` attribute should specify `rel="noopener"`"#
}

pub struct Noopener;

impl LintPass for Noopener {
    fn get_lints(&self) -> LintArray {
        lint_array![MAUD_NOOPENER]
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Noopener {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        if_chain! {
            if let Some(args) = match_marker_type(cx, expr, "element");
            if let Some((el_name, el_span)) = args.get(0).and_then(extract_strings);
            if el_name == "a";
            if let Some(attrs) = args.get(1).and_then(|expr| extract_attrs(cx, expr));
            if let Some(target_span) = has_target_but_not_noopener(&attrs);
            then {
                cx
                    .struct_span_lint(
                        MAUD_NOOPENER,
                        el_span,
                        r#"links with a `target` attribute should also specify `rel="noopener"`"#,
                    )
                    .span_label(target_span, "`target` attribute")
                    .help(concat!(
                        "for further information visit ",
                        "https://maud.lambda.xyz/lints.html#maud-noopener"))
                    .emit();
            }
        }
    }
}

fn has_target_but_not_noopener(attrs: &[(String, Span)]) -> Option<Span> {
    let mut target_span = None;
    for (at_name, at_span) in attrs {
        // TODO check for "noopener" as well
        if at_name.eq_ignore_ascii_case("rel") {
            return None;
        } else if at_name.eq_ignore_ascii_case("target") {
            target_span = Some(at_span);
        }
    }
    target_span.cloned()
}
