#![feature(plugin)]
#![plugin(rustlex)]
#[allow(plugin_as_library)]
extern crate rustlex;
#[macro_use] extern crate log;
extern crate time;
extern crate term;
extern crate getopts;

pub mod config;
pub mod frontend;
pub mod backend;
pub use backend::zcode::zfile;
pub mod utils;

use frontend::codegen;

use config::{Config,TestCase};
use std::io::{Read,Write};


#[allow(unused_variables)]
pub fn compile<R: Read, W: Write>(cfg: Config, input: &mut R, output: &mut W) {
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

#[allow(unused_variables)]
pub fn test_library<R: Read, W: Write>(cfg: Config, input: &mut Option<R>, output: &mut Option<W>) {
    for case in cfg.test_cases {
        match case {
            TestCase::ZcodeBackend => {
                match output.as_mut() {
                     Some(o) => backend::zcode::temp_create_zcode_example(o),
                     None => error!("TestCase::ZcodeBackend requires output!"),
                }
            }
        }
    }
}
