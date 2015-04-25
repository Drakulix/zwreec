pub mod frontend;
pub mod backend;
pub mod file;


//use self::file;


pub fn compile(input_file_name: &str, output_file_name: &str) {
    println!("inputFile: {}", input_file_name);
    println!("outputFile: {}", output_file_name);

    // open file
    file::open_source_file(input_file_name);

    // compile

    backend::zcode::temp_create_hello_world_zcode();
    
}

#[test]
fn it_works() {
}
