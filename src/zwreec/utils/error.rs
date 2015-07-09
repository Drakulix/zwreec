use std::fmt::{Display, Formatter, Result, Write};

use frontend::lexer::Token;
use frontend::lexer::LexerError;
use frontend::parser::ParserError;
use frontend::expressionparser::ExpressionParserError;
use frontend::evaluate_expression::EvaluateExpressionError;
use backend::codegen::CodeGenError;

macro_rules! error_panic(
    ($cfg:expr => $($arg:tt)+) => (
        {
            if !$cfg.force {
                error!("{}", $($arg)*);
                panic!("Config is set to panic at any error. Try setting the --force flag to ignore this and other errors.")
            } else {
                warn!("{}", $($arg)*);
            }
        }
    )
);

macro_rules! error_force_panic(
    ($($arg:tt)+) => (
        {
            error!("{}", $($arg)*);
            panic!("Can't continue. This error is not recoverable and not ignorable through --force.");
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

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(f.write_str("[!!!] Critical Lexer Error\n[!!!] "));
        match self {
            &LexerError::UnexpectedCharacter { character, location } => {
                try!(f.write_fmt(format_args!("Unexpected character '{}' at {}:{}", character, location.0, location.1)))
            }
        };
        Ok(())
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(f.write_str("[!!!] Critical Parser Error\n[!!!] "));
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
        try!(f.write_str("[!!!] Critical Expression Parser Error\n[!!!] "));
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

impl Display for CodeGenError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(f.write_str("[!!!] Critical Codegen Error:\n[!!!] "));
        match self {
            &CodeGenError::CouldNotWriteToOutput { ref why } => {
                try!(f.write_fmt(format_args!("Could not write to output: {}", why)))
            },
            &CodeGenError::UnsupportedExpression { ref token } => {
                try!(f.write_fmt(format_args!("Unsupported expression at {}:{}", token.location().0, token.location().1)))
            },
            &CodeGenError::UnsupportedIfExpression { ref token } => {
                try!(f.write_fmt(format_args!("Unsupported if-expression at {}:{}", token.location().0, token.location().1)))
            },
            &CodeGenError::UnsupportedElseIfExpression { ref token } => {
                try!(f.write_fmt(format_args!("Unsupported else-if-expression at {}:{}", token.location().0, token.location().1)))
            },
            &CodeGenError::UnsupportedExpressionType { ref name } => {
                try!(f.write_fmt(format_args!("This expression type is not supported right now: {}", name)))
            },
            &CodeGenError::UnsupportedLongExpression { ref name, ref token } => {
                try!(f.write_fmt(format_args!("Error at {}:{}: {} does not support any kind of expression, only variables or constants.", token.location().0, token.location().1, name)))
            },
            &CodeGenError::NoMatch { ref token } => {
                try!(f.write_fmt(format_args!("Can't find any AST operation for token: {}", token)))
            },
            &CodeGenError::PassageDoesNotExist { ref name } => {
                try!(f.write_fmt(format_args!("Referenced passage '{}' but the passage does not exist", name)))
            },
            &CodeGenError::InvalidAST => {
                try!(f.write_str("Internal error: Unexpected AST node. This should not happen. Report a bug please."))
            },
            &CodeGenError::NoStartPassage => {
                try!(f.write_str("Start passage does not exist or can not be found. Every Twee file needs a passage with the name 'Start'."))
            },
            &CodeGenError::IdentifierStackEmpty => {
                try!(f.write_str("Identifier stack is empty. Operation wasn't possible."))
            },
            &CodeGenError::SymbolMapEmpty => {
                try!(f.write_str("Symbol map is empty. Operation wasn't possible."))
            },
            &CodeGenError::CouldNotFindSymbolId => {
                try!(f.write_str("Could not find symbol ID in symbol table. Report a bug."))
            }
        };
        Ok(())
    }
}

impl Display for EvaluateExpressionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(f.write_str("[!!!] Critical Error while evaluating expression:\n[!!!] "));
        match self {
            &EvaluateExpressionError::NumericOperatorNeedsTwoArguments { ref op_name, location: (line, ch) } => {
                try!(f.write_fmt(format_args!("Numeric Operator '{}' at {}:{} needs two arguments", op_name, line, ch)))
            },
            &EvaluateExpressionError::UnsupportedOperator { ref op_name, location: (line, ch) } => {
                try!(f.write_fmt(format_args!("Operator '{}' at {}:{} is not supported right now", op_name, line, ch)))
            },
            &EvaluateExpressionError::UnsupportedFunctionArgsLen { ref name, location: (line, ch), expected } => {
                try!(f.write_fmt(format_args!("Function '{}' at {}:{} can only take {} arguments", name, line, ch, expected)))
            },
            &EvaluateExpressionError::UnsupportedFunctionArgType { ref name, index, location: (line, ch) } => {
                try!(f.write_fmt(format_args!("Function '{}' at {}:{}: Unsupported argument type at argument #{}", name, line, ch, index)))
            }
            &EvaluateExpressionError::InvalidAST => {
                try!(f.write_str("Internal error: Unsupported AST node. This should not happen. Report a bug please."));
            },
            &EvaluateExpressionError::UnsupportedFunction { ref name, location: (line, ch) } => {
                try!(f.write_fmt(format_args!("Function '{}' at {}:{} is not supported right now", name, line, ch)))
            },
            &EvaluateExpressionError::NoTempIdLeftOnStack => {
                try!(f.write_str("No temporary identifier left on the stack. Expression is too long."))
            },
            &EvaluateExpressionError::UnhandledToken { ref token } => {
                try!(f.write_fmt(format_args!("Unhandled token in expression: {:?}", token)))
            }
        };
        Ok(())
    }
}
