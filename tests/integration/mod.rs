/// Integration tests for the whole project

extern crate zwreec;
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::Cursor;
use std::vec::Vec;

static TESTFOLDER: &'static str = "./tests/integration/sample/";

fn test_compile(input_filename: String) {
	let path = Path::new(&input_filename);
    let mut input = match File::open(path) {
        Err(why) => {
            panic!("Couldn't open {}: {}",
                           path.display(), Error::description(&why))
        },
        Ok(file) => {
            file
        }
    };

    let vec: Vec<u8> = vec![];
    let mut output = Cursor::new(vec);

    zwreec::compile(&mut input, &mut output);
}

#[test]
fn helloworld_test() {
	test_compile(TESTFOLDER.to_string() + "HelloWorld.twee");
}

#[test]
fn long_text_test() {
    test_compile(TESTFOLDER.to_string() + "HelloWorld.twee");
}

#[test]
fn zscii_test() {
	test_compile(TESTFOLDER.to_string() + "ZSCII.twee");
}

#[test]
fn ascii_test() {
	test_compile(TESTFOLDER.to_string() + "ASCII.twee");
}

#[test]
fn unicode_test() {
    test_compile(TESTFOLDER.to_string() + "Unicode.twee");
}
