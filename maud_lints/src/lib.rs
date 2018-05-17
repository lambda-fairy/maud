#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(macro_vis_matcher)]
#![recursion_limit = "1000"]  // if_chain

#![doc(html_root_url = "https://docs.rs/maud_lints/0.17.4")]

#[macro_use]
extern crate if_chain;
#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;
extern crate syntax_pos;

use rustc_plugin::Registry;

#[macro_use]
mod util;

mod doctype;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(Box::new(doctype::Doctype));
    reg.register_lint_group("maud", vec![
        doctype::MAUD_DOCTYPE,
    ]);
}
