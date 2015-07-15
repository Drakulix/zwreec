//! The `expressionparser` module parses every expression
//! to an [AST (abstract syntax tree)](../ast/index.html).
//! The idea is explained here: http://programmers.stackexchange.com/questions/254074/

use config::Config;
use frontend::ast::{ASTNode, NodeDefault};
use frontend::lexer::Token;
use frontend::lexer::Token::*;

/// The errors that can occur in the ExpressionParser.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum ExpressionParserError {
    /// The operator stack is empty
    OperStackIsEmpty,

    /// The sub-expression is not parseable
    NoParseableSubExpression,

    /// There can't be more than one expression root
    MoreThanOneRootExpression { count: usize, stack: Vec<ASTNode> },

    /// Stack underflow
    NotEnoughElementsOnStack,

    /// Missing a Node
    MissingNodeForBinaryNode,

    /// Operator is unkown / invalid
    DisallowedOperator { op: Token },

    /// Operator is not implemented
    NotImplementedOperator { op: String },
}

/// Parses an expression and ASTNodes.
pub struct ExpressionParser<'a> {
    /// The stack for the expressions
    expr_stack: Vec<ASTNode>,

    /// The stack for the operators for precedence
    oper_stack: Vec<Token>,

    /// The zwreec Config
    cfg: &'a Config,
}

impl<'a> ExpressionParser<'a> {

    /// Gets node (with an expression) and starts the parsing.
    pub fn parse(node: &mut NodeDefault, cfg: &'a Config) {
        let mut expr_parser = ExpressionParser {
            expr_stack: Vec::new(),
            oper_stack: Vec::new(),
            cfg: cfg,
        };
        expr_parser.parse_expressions(node);
    }

    /// Parse the expression node and creates mutliple ast nodes.
    fn parse_expressions(&mut self, node: &mut NodeDefault) {
        node.childs.reverse();
        while let Some(top) = node.childs.pop() {
            match top.category() {
                tok @ TokBoolean  { .. } |
                tok @ TokInt      { .. } |
                tok @ TokString   { .. } |
                tok @ TokFunction { .. } |
                tok @ TokArrayLength { .. } |
                tok @ TokArrayAccess { .. } |
                tok @ TokVariable { .. } => {
                    let childs_copy = top.as_default().childs.to_vec();
                    self.expr_stack.push( ASTNode::Default(NodeDefault { category: tok.clone(), childs: childs_copy }) );
                },
                tok @ TokNumOp      { .. } |
                tok @ TokCompOp     { .. } |
                tok @ TokLogOp      { .. } |
                tok @ TokUnaryMinus { .. } => {
                    let length = self.oper_stack.len();

                    // cycle through the oper_stack stack backwards
                    // if the rank of the current operator is <= the top of the
                    // stack, we create a new node
                    // if anybody is good in rust, please refactor this. it
                    // should be:
                    // while(is_ranking_not_higher(oper_stack.top(), tok.clone())) { ...
                    for i in 0..length {
                        let i_rev = length - i - 1;
                        let token: Token = match self.oper_stack.get(i_rev) {
                            Some(tok) => tok.clone(),
                            None      => {
                                error_panic!(self.cfg => ExpressionParserError::OperStackIsEmpty);
                                return
                            }
                        };
                        if self.is_ranking_not_higher(token.clone(), tok.clone()) {
                            self.new_operator_node();
                        }

                    }

                    self.oper_stack.push(tok.clone());
                },
                tok @ TokExpression => {
                    // more ugly code.
                    // an expression-node is a child of an expression, if there
                    // where parentheses in the expression. but we don't want
                    // them, so we parse the subexpression in the parentheses

                    // make a copy of the top-node. (becouse node is borrowed)
                    // and then parse it again
                    let childs_copy = top.as_default().childs.to_vec();
                    let mut ast_node = NodeDefault { category: tok.clone(), childs: childs_copy };
                    ExpressionParser::parse(&mut ast_node, self.cfg);

                    if let Some(temp) = ast_node.childs.get(0) {
                        self.expr_stack.push(temp.clone());
                    } else {
                        error_panic!(self.cfg => ExpressionParserError::NoParseableSubExpression);
                    }
                },
                _ => ()
            }
        }

        // Parse the last elements of the stacks.
        // To avoid an endless loop we try max until stack.len()
        for _ in 0..self.expr_stack.len() {
            if self.expr_stack.len() > 0 {
                self.new_operator_node();
            }
        }

        // Multiple operators could be on the stack because of the unary operators
        for _ in 0..self.oper_stack.len() {
            if self.oper_stack.len() > 0 {
                self.new_operator_node();
            }
        }

        if self.expr_stack.len() != 1 {
            error_panic!(self.cfg => ExpressionParserError::MoreThanOneRootExpression { count: self.expr_stack.len(),
                stack: self.expr_stack.clone() });
        }

        // We are finished parsing the expression
        // Add the root of the expressions as child to the AST
        if let Some(root) = self.expr_stack.pop() {
            node.childs.push(root);
        }
    }

    /// Creates a node with an operator as the root.
    fn new_operator_node(&mut self) {
        if let Some(top_op) = self.oper_stack.pop() {

            let is_unary: bool = match top_op.clone() {
                TokLogOp { op_name: op, .. } => match &*op {
                    "not" => true,
                    "!"   => true,
                    _     => false
                },
                TokUnaryMinus { .. } => true,
                _  => false
            };

            if self.expr_stack.len() > 0 {
                let e2: ASTNode = match self.expr_stack.pop() {
                    Some(tok) => tok,
                    None      => {
                        error_panic!(self.cfg => ExpressionParserError::NotEnoughElementsOnStack);
                        return
                    }
                };

                let new_node: ASTNode;

                if is_unary {
                    new_node = ASTNode::Default(NodeDefault { category: top_op.clone(), childs: vec![e2] });
                } else {
                    let e1: ASTNode = match self.expr_stack.pop() {
                        Some(tok) => tok,
                        None      => {
                            error_panic!(self.cfg => ExpressionParserError::MissingNodeForBinaryNode);
                            return
                        }
                    };
                    new_node = ASTNode::Default(NodeDefault { category: top_op.clone(), childs: vec![e1, e2] });
                }

                self.expr_stack.push( new_node );
            } else {
                // multiple unary operators in a row like "not not true"
                self.oper_stack.push(top_op.clone());
            }
        }
    }

    /// Checks the operator precedence of two Tokens.
    ///
    /// Returns true if the operator of token1 is stronger than operator of token2.
    /// The ranking is specified in the function `operator_rank`.
    fn is_ranking_not_higher(&self, token1: Token, token2: Token) -> bool {
        let op1: String = match token1 {
            TokUnaryMinus{ .. } => "_".to_string(),
            TokNumOp     { op_name, .. } |
            TokCompOp    { op_name, .. } |
            TokLogOp     { op_name, .. } => {
                op_name.clone()
            },
            _ => {
                error_panic!(self.cfg => ExpressionParserError::DisallowedOperator { op: token1.clone() });
                return false
            }
        };
        let op2: String = match token2 {
            TokUnaryMinus{ .. } => "_".to_string(),
            TokNumOp     { op_name, .. } |
            TokCompOp    { op_name, .. } |
            TokLogOp     { op_name, .. } => {
                op_name.clone()
            },
            _ => {
                error_panic!(self.cfg => ExpressionParserError::DisallowedOperator { op: token2.clone() });
                return false
            }
        };


        // Special handling for the unary operators (two unary operators in a row)
        if (op1 == "_" || op1 == "not" || op1 == "!") &&
            self.operator_rank(op1.clone()) == self.operator_rank(op2.clone()) {

            return false
        }

        self.operator_rank(op1) >= self.operator_rank(op2)
    }

    /// Specifies the ranking of the operators.
    fn operator_rank(&self, op: String) -> u8 {
        match op.as_ref() {
            "or" | "||"         => 1,
            "and" | "&&"        => 2,
            "is" | "==" | "eq" | "!=" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte"
                                => 3,
            "+" | "-"           => 4,
            "*" | "/" | "%"     => 5,
            "_" | "not" | "!"   => 6, // _ is unary minus
            _                   => {
                error_panic!(self.cfg => ExpressionParserError::NotImplementedOperator { op: op });
                0
            }
        }
    }
}
