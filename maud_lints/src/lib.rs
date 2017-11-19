#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(macro_vis_matcher)]
#![recursion_limit = "1000"]  // if_chain

#![doc(html_root_url = "https://docs.rs/maud_lints/0.17.0")]

#[macro_use]
extern crate if_chain;
#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;

#[macro_use]
mod util;
mod maud_lint;

mod doctype;
mod frob;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(Box::new(doctype::Doctype));
    reg.register_late_lint_pass(Box::new(maud_lint::UseMaudLintPass(frob::Frob)));
    reg.register_lint_group("maud", vec![
        doctype::MAUD_DOCTYPE,
        frob::MAUD_FROB,
    ]);
}
