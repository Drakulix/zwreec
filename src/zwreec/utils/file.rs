//! This is documentation for the `file` module.

use std::io::prelude::Read;
use std::io::prelude::Write;
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
        Ok(_) => debug!("{} contains:\n-------\n{}\n----", display, content),
    }

    content
}

/// saves bytes to an file
pub fn save_bytes_to_file(target_file_name: &str, bytes: &[u8]) {
    // saving
    info!("save to file: {}", target_file_name);

    let path = Path::new(target_file_name);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           Error::description(&why)),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => {
            panic!("couldn't write to {}: {}", display,
                                               Error::description(&why))
        },
        Ok(_) => info!("successfully wrote to {}", display),
    }
}

pub fn save_target_file(target_file_name: &str) {
    // saving
    info!("save to file: {}", target_file_name)
}
