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
    // tokenize
    let tokens = frontend::lexer::lex(&cfg, input);

    //create parser
    let parser = frontend::parser::Parser::new(&cfg);

    //build up ast from tokens
    let ast = frontend::ast::AST::build(parser.parse(tokens.inspect(|ref token| {
        debug!("{:?}", token);
    })));
    ast.print();

    // create code
    codegen::generate_zcode(&cfg, ast, output);
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
