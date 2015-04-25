pub mod frontend;
pub mod backend;
pub mod file;

#[macro_use]
pub mod utils;

//use self::file;


pub fn compile(input_file_name: &str, output_file_name: &str) {
    info!("inputFile: {}", input_file_name);
    info!("outputFile: {}", output_file_name);

    // open file
    file::open_source_file(input_file_name);

    // compile


    // save file
}

#[test]
fn it_works() {
}
