#![feature(plugin)]
#![plugin(rustlex)]
#[allow(plugin_as_library)]
extern crate rustlex;
#[macro_use] extern crate log;
extern crate time;
extern crate term;

pub use backend::zcode::zfile;


pub mod frontend;
pub mod backend;
pub mod utils;
use utils::file;


pub fn compile(input_file_name: &str, output_file_name: &str) {
    info!("inputFile: {}", input_file_name);
    info!("outputFile: {}", output_file_name);

    // TODO: Uncomment when arguments are used
    // open file
    //file::open_source_file(input_file_name);

    // compile
    let input = utils::file::open_source_file(input_file_name);

    let tokens = frontend::lexer::lex(input);

    println!("");
    for token in tokens.iter() {
    	debug!("{:?}", token);
    }

    let ast = frontend::parser::parse_tokens(tokens);
    ast.print();



    let mut zfile: zfile::Zfile = zfile::Zfile::new();
    zfile.start();
    zfile.op_call_1n("main");
    zfile.op_quit();
    zfile.routine("main", 0);
    ast.to_zcode(&mut zfile);
    zfile.op_quit();
    zfile.end();
    file::save_bytes_to_file("test.z8", &(*zfile.data.bytes));





    backend::zcode::temp_create_zcode_example();

}

#[test]
fn it_works() {
}
