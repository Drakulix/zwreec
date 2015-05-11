#[macro_use] extern crate log;
extern crate time;
extern crate term;

pub mod frontend;
pub mod backend;
pub mod utils;
pub mod parsetree;


pub fn compile(input_file_name: &str, output_file_name: &str) {
    info!("inputFile: {}", input_file_name);
    info!("outputFile: {}", output_file_name);

    // TODO: Uncomment when arguments are used
    // open file
    //file::open_source_file(input_file_name);

    // compile
    parsetree::temp_create_syntax_tree();

    backend::zcode::temp_create_zcode_example();

}

#[test]
fn it_works() {
}
