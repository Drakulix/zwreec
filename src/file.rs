//! This is documentation for the `file` module.
//!
//! # Examples


use std::io::prelude::Read;
use std::error::Error;
use std::fs::File;
use std::fs;
use std::path::Path;


/// Read Text File
///
/// # Examples
///
/// ```ignore
/// use zwreec::file;
///
/// let stringContent = file::open_source_file("../README.md");
/// ```
pub fn open_source_file(source_file_name: &str) -> String {

    let path = Path::new(source_file_name);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   Error::description(&why)),
        Ok(file) => file,
    };

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                   Error::description(&why)),
        Ok(_) => print!("{} contains:\n-------\n{}\n----\n", display, content),
    }

    content
}

pub fn save_target_file(target_file_name: &str) {
    // saving
    println!("save to file: {}", target_file_name)
}

pub fn temp_print_current_directory_files() {
    let paths = fs::read_dir(&Path::new(".")).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
}

pub fn temp_hello() -> String {
    "hello from file".to_string()
}

#[test]
fn it_works() {
    assert!(true);
}
