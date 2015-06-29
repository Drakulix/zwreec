use std::fmt::{Display, Formatter, Result, Write};

use frontend::lexer::Token;
use frontend::parser::ParserError;
use frontend::expressionparser::ExpressionParserError;

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
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(f.write_str("[!!!] Critical Parser Error [!!!]\n"));
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

impl Display for ExpressionParserError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(f.write_str("[!!!] Critical Expression Parser Error [!!!]\n"));
        match self {
            &ExpressionParserError::OperStackIsEmpty => {
                try!(f.write_str("No token in the operators stack."))
            },
            &ExpressionParserError::NoParseableSubExpression => {
                try!(f.write_str("No parsable sub-expression"))
            },
            &ExpressionParserError::MoreThanOneRootExpression { count, ref stack } => {
                try!(f.write_fmt(format_args!("Only one expression can be the root. But there are {}. Stack: {:?}", count, stack)))
            },
            &ExpressionParserError::NotEnoughElementsOnStack => {
                try!(f.write_str("Not enough elements on the stack to create a node"))
            },
            &ExpressionParserError::MissingNodeForBinaryNode => {
                try!(f.write_str("Missing Node to create binary node"))
            },
            &ExpressionParserError::DisallowedOperator { ref op } => {
                try!(f.write_fmt(format_args!("Checking the operator ranking for operator '{:?}' failed: The operator is not allowed in this context.", op)))
            },
            &ExpressionParserError::NotImplementedOperator { ref op } => {
                try!(f.write_fmt(format_args!("This operator is not implemented: '{}'", op)))
            }
        };
        Ok(())
    }
}