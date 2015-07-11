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

    let outvec = output.into_inner();

    // check that the z-code version is 8
    // this ensures that at least some z-code was emitted
    assert_eq!(0x08, outvec[0]);
}

#[test]
fn expressions_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "Expressions.twee");
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
fn prompt_and_string_index_access_test() {
    test_compile(TESTFOLDER_PASS.to_string() + "textadventure.twee");
}

#[test]
#[should_panic]
fn expression_double_operators_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "ExpressionDoubleOperators.twee");
}

#[test]
#[should_panic]
fn expression_wrong_parentheses1_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "ExpressionsWrongParentheses1.twee");
}

#[test]
#[should_panic]
fn expression_wrong_parentheses2_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "ExpressionsWrongParentheses2.twee");
}

#[test]
#[should_panic]
fn duplicate_passages() {
    test_compile(TESTFOLDER_FAIL.to_string() + "DuplicatePassage.twee");
}

#[test]
#[should_panic]
fn invalid_macro_test() {
   test_compile(TESTFOLDER_FAIL.to_string() + "InvalidMacro.twee");
}

#[test]
#[should_panic]
fn invalid_passage_link() {
    test_compile(TESTFOLDER_FAIL.to_string() + "InvalidPassageLink.twee");
}

#[test]
#[should_panic]
fn multiple_else_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "MultipleElse.twee");
}

#[test]
#[should_panic]
fn multiple_endif_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "MultipleEndIf.twee");
}

#[test]
#[should_panic]
fn no_start_passage_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "NoStartPassage.twee");
}

#[test]
#[should_panic]
fn passage_not_allowed_chars1_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "PassageNotAllowedChars1.twee");
}

#[test]
#[should_panic]
fn passage_not_allowed_chars2_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "PassageNotAllowedChars2.twee");
}

#[test]
#[should_panic]
fn wrong_formatting_test() {
    test_compile(TESTFOLDER_FAIL.to_string() + "WrongFormatting.twee");
}
