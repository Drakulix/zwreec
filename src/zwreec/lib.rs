#![feature(plugin)]
#![plugin(rustlex)]
#[allow(plugin_as_library)]
extern crate rustlex;
#[macro_use] extern crate log;
extern crate time;
extern crate term;

pub mod frontend;
pub mod backend;
pub use backend::zcode::zfile;
pub mod utils;

use frontend::codegen;

use std::io::{Read,Write};


pub fn compile<R: Read, W: Write>(input: &mut R, output: &mut W) {
    // compile

    //screen
    let mut clean_input = frontend::screener::screen(input);

    // tokenize
    let tokens = frontend::lexer::lex(&mut clean_input);

    // parse tokens and create ast
    let ast = frontend::parser::parse_tokens(tokens.inspect(|ref token| {
        debug!("{:?}", token);
    }).collect()); //use collect until we work on iterators directly
    ast.print();

    // create code
    codegen::generate_zcode(ast, output);
}
