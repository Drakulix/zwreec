pub mod lexer;
pub mod parser;
pub mod ast;
pub mod parsetree;
pub mod codegen;

pub fn lex<T: Iterator>(input :String)-> Vec<lexer::Token> {
    lexer::lex(input)
}

/// only temp-method, tests/lib.rs checks it
pub fn temp_hello() -> String {
    "hello from frontend".to_string()
}


#[test]
fn it_works() {

}
