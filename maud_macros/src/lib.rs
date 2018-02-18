#![feature(proc_macro)]

#![doc(html_root_url = "https://docs.rs/maud_macros/0.17.2")]

extern crate literalext;
#[macro_use] extern crate matches;
extern crate maud_htmlescape;
extern crate proc_macro;

mod ast;
mod generate;
mod parse;

use proc_macro::{Literal, Span, Term, TokenNode, TokenStream, TokenTree};
use proc_macro::quote;

type ParseResult<T> = Result<T, String>;

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    expand(input)
}

#[proc_macro]
pub fn html_debug(input: TokenStream) -> TokenStream {
    let expr = expand(input);
    println!("expansion:\n{}", expr);
    expr
}

fn expand(input: TokenStream) -> TokenStream {
    let output_ident = TokenTree {
        kind: TokenNode::Term(Term::intern("__maud_output")),
        span: Span::def_site(),
    };
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let size_hint = TokenNode::Literal(Literal::u64(size_hint as u64));
    let markups = match parse::parse(input) {
        Ok(markups) => markups,
        Err(e) => panic!(e),
    };
    let stmts = generate::generate(markups, output_ident.clone());
    quote!({
        extern crate maud;
        let mut $output_ident = String::with_capacity($size_hint as usize);
        $stmts
        maud::PreEscaped($output_ident)
    })
}
