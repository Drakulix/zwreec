use std::fmt::{Display, Formatter, Result, Write};

use frontend::lexer::Token;
use frontend::parser::ParserError;

macro_rules! error_panic(
    ($cfg:expr => $($arg:tt)+) => (
        {
            error!("{}", $($arg)*);
            if !$cfg.force {
                panic!()
            }
        }
    )
);

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &Token::TokError{ref message, ..} => try!(f.write_str(&*message)),
            _ => try!(f.write_fmt(format_args!("{:?}", self))),
        };
        Ok(())
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> Result{
        try!(f.write_str("[!!!] Critical Parser Error [!!!]"));
        match self {
            &ParserError::TokenDoNotMatch{ref token, ref stack} =>
                match token {
                    &Some(ref token) => try!(f.write_fmt(format_args!("Stack Token does not match. Token:{:?} Stack:{:?}", token, stack))),
                    &None => try!(f.write_fmt(format_args!("No Tokens left, but Terminal is left on Stack. Token:{:?}", stack))),
                },
            &ParserError::StackIsEmpty{ref token} => try!(f.write_fmt(format_args!("Tokens left but Stack is empty. Token:{:?}", token))),
            &ParserError::NoProjection{ref token, ref stack} => try!(f.write_fmt(format_args!("No Projection found for Token:{:?} and NonTerminal:{:?}", token, stack))),
            &ParserError::NonTerminalEnd{ref stack} => try!(f.write_fmt(format_args!("NonTerminal:{:?} is no allowed End", stack))),
        };
        Ok(())
    }
}
