pub mod lexer;
pub mod parser;
pub mod ast;
pub mod parsetree;
pub mod codegen;

pub fn lex<T: Iterator>(input :String)-> Vec<lexer::Token> {
    lexer::lex(input)
}


#[test]
fn it_works() {

}
