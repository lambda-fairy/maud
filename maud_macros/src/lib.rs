#![feature(proc_macro)]

#![doc(html_root_url = "https://docs.rs/maud_macros/0.17.2")]

extern crate literalext;
#[macro_use] extern crate matches;
extern crate maud_htmlescape;
extern crate proc_macro;

mod ast;
mod build;
mod generate;
mod parse;

use proc_macro::{Literal, Span, Term, TokenNode, TokenStream, TokenTree};
use proc_macro::quote;

type ParseResult<T> = Result<T, String>;

use parse::{BufferType, OutputBuffer};

enum OutputType {
    NewString,
    ProvidedBuffer
}

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    expand(input, OutputType::NewString)
}

#[proc_macro]
pub fn html_to(input: TokenStream) -> TokenStream {
    expand(input, OutputType::ProvidedBuffer)
}

#[proc_macro]
pub fn html_debug(input: TokenStream) -> TokenStream {
    let expr = html(input);
    println!("expansion of html!:\n{}", expr);
    expr
}

#[proc_macro]
pub fn html_to_debug(input: TokenStream) -> TokenStream {
    let expr = html_to(input);
    println!("expansion of html_to!:\n{}", expr);
    expr
}

fn expand(mut input: TokenStream, output_type: OutputType) -> TokenStream {
    let output_buffer = match output_type {
        OutputType::NewString => OutputBuffer::new(
            TokenTree {
                kind: TokenNode::Term(Term::intern("__maud_output")),
                span: Span::def_site(),
            },
            BufferType::Allocated,
        ),
        OutputType::ProvidedBuffer => match parse::buffer_argument(&mut input) {
            Ok(output_buffer) => output_buffer,
            Err(e) => panic!(e),
        },
    };
    // Heuristic: the size of the resulting markup tends to correlate with the
    // code size of the template itself
    let size_hint = input.to_string().len();
    let size_hint = TokenNode::Literal(Literal::u64(size_hint as u64));
    let markups = match parse::parse(input) {
        Ok(markups) => markups,
        Err(e) => panic!(e),
    };
    let stmts = generate::generate(markups, output_buffer.clone());
    match output_type {
        OutputType::ProvidedBuffer => quote!({
            extern crate maud;
            $stmts
        }),
        OutputType::NewString => {
            let output_ident = output_buffer.ident();
            quote!({
                extern crate maud;
                let mut $output_ident = String::with_capacity($size_hint as usize);
                $stmts
                maud::PreEscaped($output_ident)
            })
        }
    }
}
