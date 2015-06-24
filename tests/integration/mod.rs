/// Integration tests for the whole project

extern crate zwreec;
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::Cursor;
use std::vec::Vec;

static TESTFOLDER_PASS: &'static str = "./tests/integration/should-compile/";
static TESTFOLDER_FAIL: &'static str = "./tests/integration/should-fail/";

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

    let cfg = zwreec::config::Config::default_config();

    zwreec::compile(cfg, &mut input, &mut output);
}

#[test]
fn helloworld_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "HelloWorld.twee");
}

#[test]
fn long_text_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "HelloWorld.twee");
}

#[test]
fn zscii_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "ZSCII.twee");
}

#[test]
fn ascii_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "ASCII.twee");
}

#[test]
fn unicode_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "Unicode.twee");
}

#[test]
fn passage_links_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "PassageLinks.twee");
}

#[test]
fn random_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "Random.twee");
}

#[test]
fn random_expanded_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "RandomExpanded.twee");
}

#[test]
fn if_else_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "If-Else.twee");
}

#[test]
fn current_status_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "CurrentStatus.twee");
}

#[test]
#[should_panic]
fn invalid_macro_test() {
   test_compile(TESTFOLDER_FAIL.to_string() + "InvalidMacro.twee");
}

#[test]
#[should_panic]
fn no_start_passage_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "NoStartPassage.twee");
}

#[test]
#[should_panic]
fn duplicate_passage_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "DuplicatePassage.twee");
}
