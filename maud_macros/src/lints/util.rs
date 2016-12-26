//! Miscellaneous utilities for writing lints.
//!
//! Most of these are adapted from Clippy.

use rustc::hir::def_id::DefId;
use rustc::lint::LateContext;
use rustc::ty;
use syntax::symbol::{InternedString, Symbol};

/// Produce a nested chain of if-lets and ifs from the patterns:
///
/// ```rust,ignore
/// if_let_chain! {[
///     let Some(y) = x,
///     y.len() == 2,
///     let Some(z) = y,
/// ], {
///     block
/// }}
/// ```
///
/// becomes
///
/// ```rust,ignore
/// if let Some(y) = x {
///     if y.len() == 2 {
///         if let Some(z) = y {
///             block
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! if_let_chain {
    ([let $pat:pat = $expr:expr, $($tt:tt)+], $block:block) => {
        if let $pat = $expr {
           if_let_chain!{ [$($tt)+], $block }
        }
    };
    ([let $pat:pat = $expr:expr], $block:block) => {
        if let $pat = $expr {
           $block
        }
    };
    ([let $pat:pat = $expr:expr,], $block:block) => {
        if let $pat = $expr {
           $block
        }
    };
    ([$expr:expr, $($tt:tt)+], $block:block) => {
        if $expr {
           if_let_chain!{ [$($tt)+], $block }
        }
    };
    ([$expr:expr], $block:block) => {
        if $expr {
           $block
        }
    };
    ([$expr:expr,], $block:block) => {
        if $expr {
           $block
        }
    };
}

/// Check if a `DefId`'s path matches the given absolute type path usage.
///
/// # Examples
/// ```rust,ignore
/// match_def_path(cx, id, &["core", "option", "Option"])
/// ```
pub fn match_def_path(cx: &LateContext, def_id: DefId, path: &[&str]) -> bool {
    struct AbsolutePathBuffer {
        names: Vec<InternedString>,
    }

    impl ty::item_path::ItemPathBuffer for AbsolutePathBuffer {
        fn root_mode(&self) -> &ty::item_path::RootMode {
            const ABSOLUTE: &'static ty::item_path::RootMode = &ty::item_path::RootMode::Absolute;
            ABSOLUTE
        }

        fn push(&mut self, text: &str) {
            self.names.push(Symbol::intern(text).as_str());
        }
    }

    let mut apb = AbsolutePathBuffer { names: vec![] };
    cx.tcx.push_item_path(&mut apb, def_id);
    apb.names.len() == path.len() && apb.names.iter().zip(path.iter()).all(|(a, &b)| &**a == b)
}
