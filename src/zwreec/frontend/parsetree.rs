//! The `parsetree` module contains a lot of useful functionality
//! to create and walk through the parsetree

use frontend::lexer::Token;
use frontend::parser::{NonTerminalType};


//#[derive(Debug, Clone)]
pub enum PNode {
    NonTerminal (NonTerminalType),
    Terminal (Token)
}

impl PNode {
    pub fn new_terminal(terminal: Token) -> PNode {
        PNode::Terminal(terminal)
    }

    pub fn new_non_terminal(non_terminal: NonTerminalType) -> PNode {
        PNode::NonTerminal(non_terminal)
    }
}
