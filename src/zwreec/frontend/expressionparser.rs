//! The `expressionparser` module parses every expression
//! to an AST (abstract syntax tree).
//! The idea is explained here: http://programmers.stackexchange.com/questions/254074/

//use frontend::ast::*;
use frontend::ast::{ASTNode, NodeDefault};
use frontend::lexer::Token;
use frontend::lexer::Token::*;


pub struct ExpressionParser {
    expr_stack: Vec<ASTNode>,
    oper_stack: Vec<Token>,
}

impl ExpressionParser {
    pub fn parse(node: &mut NodeDefault) {
        let mut expr_parser = ExpressionParser {
            expr_stack: Vec::new(),
            oper_stack: Vec::new(),
        };
        expr_parser.parse_expressions(node);
    }

    /// parse the expression node and creates mutliple ast nodes
    fn parse_expressions(&mut self, node: &mut NodeDefault) {

        node.childs.reverse();
        while let Some(top) = node.childs.pop() {
            match top.category() {
                tok @ TokBoolean  { .. } |
                tok @ TokInt      { .. } |
                tok @ TokString   { .. } |
                tok @ TokFunction { .. } |
                tok @ TokVariable { .. } => {
                    let childs_copy = top.as_default().childs.to_vec();
                    self.expr_stack.push( ASTNode::Default(NodeDefault { category: tok.clone(), childs: childs_copy }) );
                },
                tok @ TokNumOp  { .. } |
                tok @ TokCompOp { .. } |
                tok @ TokLogOp  { .. } => {
                    let length = self.oper_stack.len();

                    // cycle through the oper_stack stack backwards
                    // if the rank of the current operator is <= the top of the
                    // stack, we create a new node
                    for i in 0..length {
                        let i_rev = length - i - 1;
                        let token: Token = self.oper_stack.get(i_rev).unwrap().clone();
                        if is_ranking_not_higher(token, tok.clone()) {
                            self.new_operator_node();
                        }
                    }

                    self.oper_stack.push(tok.clone());
                },
                _ => ()
            }
        }

        // parse the last elements of the stack
        // to avoid endless loop we try max expr_stack.len()
        for _ in 0..self.expr_stack.len() {
            if self.expr_stack.len() > 0 {
                self.new_operator_node();
            }
        }

        // finished. so add the root of the expressions as child.
        assert!{self.expr_stack.len() == 1, "Only one expression can be the root. But there are {:?}.", self.expr_stack.len()};
        if let Some(root) = self.expr_stack.pop() {
            node.childs.push(root);
        }
    }

    /// creates a node with an operator on top
    fn new_operator_node(&mut self) {
        if let Some(top_op) = self.oper_stack.pop() {
            let e2: ASTNode = self.expr_stack.pop().unwrap();
            let e1: ASTNode = self.expr_stack.pop().unwrap();

            let new_node = ASTNode::Default(NodeDefault { category: top_op.clone(), childs: vec![e1, e2] });
            self.expr_stack.push( new_node );
        }
    }
}

/// checks the operatores of two tokens returns true if operator of token1
/// is more important then operator of token2
/// the ranking is set in "operator_precedence"
fn is_ranking_not_higher(token1: Token, token2: Token) -> bool {
    let mut op1: String = "".to_string();
    let mut op2: String = "".to_string();
    match token1 {
        TokNumOp  { op_name, .. } |
        TokCompOp { op_name, .. } |
        TokLogOp  { op_name, .. } => {
            op1 = op_name.clone();
        },
        _ => ()
    }
    match token2 {
        TokNumOp  { op_name, .. } |
        TokCompOp { op_name, .. } |
        TokLogOp  { op_name, .. } => {
            op2 = op_name.clone();
        },
        _ => ()
    }

    if operator_rank(op1) >= operator_rank(op2) {
        return true
    }

    false
}

/// ranking of the operators
fn operator_rank(op: String) -> u8 {
    match op.as_ref() {
        "or"                => 1,
        "and"               => 2,
        "is" | "==" | "eq" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte"
                            => 3,
        "+" | "-"           => 4,
        "*" | "/" | "%"     => 5,
        "not"               => 6,
        _                   => panic!{"This operator is not implemented"}
    }
}
