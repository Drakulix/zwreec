pub mod frontend;
pub mod backend;
pub mod file;

#[macro_use]
pub mod utils;


pub fn compile(input_file_name: &str, output_file_name: &str) {
    log_info!("inputFile: {}", input_file_name);
    log_info!("outputFile: {}", output_file_name);

    // TODO: Uncomment when arguments are used
    // open file
    //file::open_source_file(input_file_name);

    // compile

    backend::zcode::temp_create_zcode_example();
    
}

#[test]
fn it_works() {
}
