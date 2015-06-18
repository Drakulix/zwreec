use frontend::lexer::LexerError;
use frontend::parser::ParserError;

trait Error {
    fn raise(self) -> ! {
        self.log();
        panic!();
    }
    fn log(self);
}

impl Error for LexerError {
    fn log(self) {
        error!("[!!!] Critical Lexer Error [!!!]");
    }
}

impl Error for ParserError {
    fn log(self) {
        error!("[!!!] Critical Parser Error [!!!]");
        match self {
            TokenDoNotMatch(token, stack) =>
                match token {
                    Some(token) => error!("Stack Token does not match. Token:{:?} Stack:{:?}", token, stack),
                    None => error!("No Tokens left, but Terminal is left on Stack. Token:{:?}", stack),
                },
            StackIsEmpty(token) => error!("Tokens left but Stack is empty. Token:{:?}", token),
        }
    }
}
