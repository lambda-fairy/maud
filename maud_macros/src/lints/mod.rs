use rustc_plugin::Registry;

#[macro_use]
mod util;

mod doctype;

pub fn register_lints(reg: &mut Registry) {
    reg.register_late_lint_pass(Box::new(doctype::Doctype));

    reg.register_lint_group("maud", vec![
        doctype::MAUD_DOCTYPE,
    ]);
}
