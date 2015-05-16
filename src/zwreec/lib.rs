#![feature(plugin)]
#![plugin(rustlex)]
#[allow(plugin_as_library)]
extern crate rustlex;
#[macro_use]
extern crate log;
extern crate time;
extern crate term;

pub mod frontend;
pub mod backend;
pub mod utils;

use frontend::codegen;

// only for the temp_create_zcode_example-method
//use backend::zcode::zfile;


pub fn compile(input_file_name: &str, output_file_name: &str) {
    info!("inputFile: {}", input_file_name);
    info!("outputFile: {}", output_file_name);

    // compile

    // read file
    let input = utils::file::open_source_file(input_file_name);

    // tokenize
    let tokens = frontend::lexer::lex(input);

    println!("");
    for token in tokens.iter() {
    	debug!("{:?}", token);
    }

    // parse tokens and create ast
    let ast = frontend::parser::parse_tokens(tokens);
    ast.print();

    // create code
    codegen::generate_zcode(ast, output_file_name);

    
    //backend::zcode::temp_create_zcode_example();
}

#[test]
fn it_works() {
}
