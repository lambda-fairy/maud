//! Miscellaneous utilities for writing lints.
//!
//! Most of these are adapted from Clippy.

use rustc::hir::def_id::DefId;
use rustc::lint::LateContext;
use rustc::ty;
use syntax::symbol::{InternedString, Symbol};

#[macro_export]
macro_rules! if_chain {
    (let $pat:pat = $expr:expr; $($tt:tt)+) => {
        {
            let $pat = $expr;
            if_chain! { $($tt)+ }
        }
    };
    (if let $pat:pat = $expr:expr; $($tt:tt)+) => {
        if let $pat = $expr {
            if_chain! { $($tt)+ }
        }
    };
    (if $expr:expr; $($tt:tt)+) => {
        if $expr {
            if_chain! { $($tt)+ }
        }
    };
    ($expr:expr) => {
        $expr
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
