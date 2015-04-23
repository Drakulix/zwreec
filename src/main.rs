// only becouse without the following line there will be warnings
// while testing :(
#![allow(dead_code)]

use std::env;

extern crate zwreec;


fn main() {
    println!("#main started");

    // handling commandline parameters
    let args: Vec<String> = env::args().collect();

    let input_file_name: &str;
    let output_file_name: &str;
    
    if args.len() == 1 {
        input_file_name = "README.md";
        output_file_name = "file_does_not_exist.zcode";
    } else if args.len() == 3 {
        println!("argument 1 {}", args[1]);
        println!("argument 2 {}", args[2]);

        input_file_name = &args[1];
        output_file_name = &args[2];
    } else {
        help();
        return;
    }

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