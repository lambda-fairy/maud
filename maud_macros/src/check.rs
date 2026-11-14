use proc_macro2_diagnostics::{Diagnostic, SpanDiagnosticExt};
use syn::spanned::Spanned;

use crate::ast::{
    Block, ControlFlow, ControlFlowKind, Element, ElementBody, IfExpr, IfOrBlock, Markup, Markups,
};

/// The void elements as defined by the HTML Living Standard.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// Checks whether each element is closed in a valid way. Non-void elements
/// must be closed with `{ ... }`, and void elements with `;`.
///
/// Emits a diagnostic for each mismatch found.
pub fn check_elements(markups: &Markups<Element>, diagnostics: &mut Vec<Diagnostic>) {
    for markup in &markups.markups {
        check_markup(markup, diagnostics);
    }
}

fn check_markup(markup: &Markup<Element>, diagnostics: &mut Vec<Diagnostic>) {
    match markup {
        Markup::Block(block) => check_block(block, diagnostics),
        Markup::Element(element) => {
            check_element(element, diagnostics);
            if let ElementBody::Block(block) = &element.body {
                check_block(block, diagnostics);
            }
        }
        Markup::ControlFlow(control_flow) => check_control_flow(control_flow, diagnostics),
        Markup::Lit(_) | Markup::Splice { .. } | Markup::Semi(_) => {}
    }
}

fn check_block(block: &Block<Element>, diagnostics: &mut Vec<Diagnostic>) {
    check_elements(&block.markups, diagnostics);
}

fn check_element(element: &Element, diagnostics: &mut Vec<Diagnostic>) {
    let name = element
        .name
        .as_ref()
        .map(|name| name.to_string().to_lowercase())
        .unwrap_or_else(|| "div".to_owned());

    match &element.body {
        ElementBody::Void(semi) if !VOID_ELEMENTS.contains(&name.as_str()) => diagnostics.push(
            semi.span()
                .error(format!(
                    "`<{name}>` is not a void element, so it cannot be closed with `;`"
                ))
                .help("change this to `{}`"),
        ),
        ElementBody::Block(block) if VOID_ELEMENTS.contains(&name.as_str()) => diagnostics.push(
            block
                .brace_token
                .span
                .open()
                .error(format!(
                    "`<{name}>` is a void element and cannot have a closing tag"
                ))
                .help("change this to `;`"),
        ),
        _ => {}
    }
}

fn check_control_flow(control_flow: &ControlFlow<Element>, diagnostics: &mut Vec<Diagnostic>) {
    match &control_flow.kind {
        ControlFlowKind::Let(_) => {}
        ControlFlowKind::If(if_) => check_if(if_, diagnostics),
        ControlFlowKind::For(for_) => check_block(&for_.body, diagnostics),
        ControlFlowKind::While(while_) => check_block(&while_.body, diagnostics),
        ControlFlowKind::Match(match_) => {
            for arm in &match_.arms {
                check_markup(&arm.body, diagnostics);
            }
        }
    }
}

fn check_if(if_: &IfExpr<Element>, diagnostics: &mut Vec<Diagnostic>) {
    check_block(&if_.then_branch, diagnostics);
    if let Some((_, _, else_branch)) = &if_.else_branch {
        match &**else_branch {
            IfOrBlock::If(if_) => check_if(if_, diagnostics),
            IfOrBlock::Block(block) => check_block(block, diagnostics),
        }
    }
}
