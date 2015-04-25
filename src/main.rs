use std::env;

#[macro_use]
pub mod utils;

extern crate zwreec;


fn main() {
    println!("#main started");

    // handling commandline parameters
    let args: Vec<String> = env::args().collect();

    let mut input_file_name: &str;
    let mut output_file_name: &str;

    match args.len() {
        1 => {
            input_file_name = "a.in";
            output_file_name = "a.out";
        },
        3 => {
            input_file_name = &args[1];
            output_file_name = &args[2];
        },
        _ => {
            help();
            return;
        }
    };

    // call library
    zwreec::compile(input_file_name, output_file_name);

    // only for testing
    println!("(1) {}", zwreec::frontend::temp_hello());
    println!("(2) {}", zwreec::backend::temp_hello());
    println!("(3) {}", zwreec::file::temp_hello());

    println!("#main finished");
}

fn help() {
    println!("usage:\n    zwreec <input_file> <output_file>");
}
