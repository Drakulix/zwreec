use frontend::lexer::LexerError;
use frontend::parser::ParserError;

pub trait Error {
    fn raise(&self) -> ! {
        self.log();
        panic!();
    }
    fn log(&self);
}

impl Error for LexerError {
    fn log(&self) {
        error!("[!!!] Critical Lexer Error [!!!]");
    }
}

impl Error for ParserError {
    fn log(&self) {
        error!("[!!!] Critical Parser Error [!!!]");
        match self {
            &ParserError::TokenDoNotMatch{ref token, ref stack} =>
                match token {
                    &Some(ref token) => error!("Stack Token does not match. Token:{:?} Stack:{:?}", token, stack),
                    &None => error!("No Tokens left, but Terminal is left on Stack. Token:{:?}", stack),
                },
            &ParserError::StackIsEmpty{ref token} => error!("Tokens left but Stack is empty. Token:{:?}", token),
            &ParserError::NoProjection{ref token, ref stack} => error!("No Projection found for Token:{:?} and NonTerminal:{:?}", token, stack),
            &ParserError::NonTerminalEnd{ref stack} => error!("NonTerminal:{:?} is no allowed End", stack),
        }
    }
}
