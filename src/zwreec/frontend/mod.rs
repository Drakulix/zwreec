pub mod lexer;

pub fn lex<T: Iterator>(input :String)-> Vec<lexer::Token> {
    lexer::lex(input)
}


#[test]
fn it_works() {

}
