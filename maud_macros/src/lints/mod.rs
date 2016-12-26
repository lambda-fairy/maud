use rustc_plugin::Registry;

#[macro_use]
mod util;

pub mod doctype_html;

pub fn register_lints(reg: &mut Registry) {
    reg.register_late_lint_pass(Box::new(doctype_html::DoctypeHtml));

    reg.register_lint_group("maud", vec![
        doctype_html::MAUD_DOCTYPE_HTML,
    ]);
}
