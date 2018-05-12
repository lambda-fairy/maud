//! Miscellaneous utilities for writing lints.
//!
//! Most of these are adapted from Clippy.

use rustc::hir::{
    Expr,
    ExprBlock,
    ExprCall,
    ExprLit,
    ExprPath,
    StmtSemi,
};
use rustc::hir::def_id::DefId;
use rustc::lint::LateContext;
use rustc::ty;
use syntax::ast::{LitKind, StrStyle};
use syntax_pos::Span;
use syntax_pos::symbol::{LocalInternedString, Symbol};

pub fn match_marker_type<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    expr: &'tcx Expr,
    marker_type: &'static str,
) -> Option<&'tcx [Expr]> {
    if_chain! {
        if let ExprCall(ref path_expr, ref args) = expr.node;
        if let ExprPath(ref qpath) = path_expr.node;
        let def_id = cx.tables.qpath_def(qpath, path_expr.hir_id).def_id();
        if match_def_path(cx, def_id, &["maud", "marker", marker_type]);
        then {
            Some(args)
        } else {
            None
        }
    }
}

/// Check if a `DefId`'s path matches the given absolute type path usage.
///
/// # Examples
/// ```rust,ignore
/// match_def_path(cx, id, &["core", "option", "Option"])
/// ```
pub fn match_def_path(cx: &LateContext, def_id: DefId, path: &[&str]) -> bool {
    struct AbsolutePathBuffer {
        names: Vec<LocalInternedString>,
    }

    impl ty::item_path::ItemPathBuffer for AbsolutePathBuffer {
        fn root_mode(&self) -> &ty::item_path::RootMode {
            &ty::item_path::RootMode::Absolute
        }

        fn push(&mut self, text: &str) {
            self.names.push(Symbol::intern(text).as_str());
        }
    }

    let mut apb = AbsolutePathBuffer { names: vec![] };
    cx.tcx.push_item_path(&mut apb, def_id);
    apb.names.len() == path.len() && apb.names.iter().zip(path.iter()).all(|(a, &b)| &**a == b)
}

pub fn match_string(expr: &Expr) -> Option<(Symbol, Span)> {
    if_chain! {
        if let ExprLit(ref lit) = expr.node;
        if let LitKind::Str(s, StrStyle::Cooked) = lit.node;
        then {
            Some((s, lit.span))
        } else {
            None
        }
    }
}

pub fn extract_attrs<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    expr: &'tcx Expr,
) -> Option<Vec<(Symbol, Span)>> {
    let block = if let ExprBlock(ref block) = expr.node {
        block
    } else {
        return None;
    };
    Some(block.stmts.iter().filter_map(|stmt| if_chain! {
        if let StmtSemi(ref expr, _) = stmt.node;
        if let Some(args) = match_marker_type(cx, expr, "attribute");
        then {
            args.get(0).and_then(match_string)
        } else {
            None
        }
    }).collect())
}
