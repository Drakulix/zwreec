//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

use std::fmt::{Debug, Formatter, Result, Write};
use std::iter::Scan;

use config::Config;
use frontend::expressionparser;
use frontend::lexer::Token;
use frontend::lexer::Token::{TokMacroIf, TokMacroElseIf, TokExpression};

use ::utils::extensions::{Constructor, ConstructorExt, Peeking, PeekingExt};

/// This is the state of the AST building operation.
pub struct ASTBuilder {
    path: Vec<usize>,
    is_in_if_expression: bool
}

/// The [parser](../parser/index.html) uses these ASTOperations to create the AST.
#[derive(Clone)]
pub enum ASTOperation {
    /// Adds a passage to the path in the ast.
    AddPassage(Token),

    /// Adds a child to the path in the ast.
    AddChild(Token),

    /// Adds a child and adds the child to the current path.
    ChildDown(Token),

    /// Adds a child and goes one level up.
    ChildUp(Token),

    /// Goes two levels up.
    TwoUp,

    /// Goes one level up in the path.
    Up,

    /// Goes one level up and adds a child.
    UpChild(Token),

    /// Goes one level up, adds a child and adds the added child to the path.
    UpChildDown(Token),

    /// This goes one level up or goes out of an if-expression if possible.
    UpSpecial,
}

impl ASTBuilder {

    /// Creates a new ASTBuilder.
    fn new() -> ASTBuilder {
        ASTBuilder {
            path: Vec::new(),
            is_in_if_expression: false
        }
    }

    /// Constructs an iterator over all passages.
    ///
    // This has a concrete return type because of optimization reasons. Read and use it as "Iterator<ASTNode>".
    pub fn build<I: Iterator<Item=ASTOperation>>(cfg: Config, ops: I)
        -> Scan<Constructor<Peeking<I, ASTOperation>, ASTBuilder, ASTNode, fn(&mut ASTBuilder, &mut Option<ASTNode>, (ASTOperation, Option<ASTOperation>)) -> Option<ASTNode>>, Config, fn(&mut Config, ASTNode) -> Option<ASTNode>>
    {
        info!("Started to build AST");

        ops.peeking().construct_state(ASTBuilder::new(),
        {
            fn construct(builder: &mut ASTBuilder, passage: &mut Option<ASTNode>, op_peek: (ASTOperation, Option<ASTOperation>)) -> Option<ASTNode>
            {
                let op_node = builder.operation(passage, op_peek.0);

                if op_peek.1.is_none() {
                    info!("Finished building AST");
                }

                op_node
            }
            construct as fn(&mut ASTBuilder, &mut Option<ASTNode>, (ASTOperation, Option<ASTOperation>)) -> Option<ASTNode> //necessary cast is a known bug
        }).scan(cfg,
            {
                fn scan(cfg: &mut Config, x: ASTNode) -> Option<ASTNode>
                {
                    let mut y = x.clone();
                    y.parse_expressions(cfg);
                    Some(y)
                }
                scan
            }
        )
    }

    /// Calls the matching function to a given ASTOperation.
    pub fn operation(&mut self, current_passage: &mut Option<ASTNode>, op: ASTOperation) -> Option<ASTNode> {
        use self::ASTOperation::*;
        match op {
            AddPassage(passage) => self.add_passage(passage),
            AddChild(child) => self.add_child(current_passage, child),
            ChildDown(child) => self.child_down(current_passage, child),
            ChildUp(child) => self.child_up(current_passage, child),
            Up => self.up(),
            UpChild(child) => self.up_child(current_passage, child),
            UpChildDown(child) => self.up_child_down(current_passage, child),
            UpSpecial => self.up_special(),
            TwoUp => self.two_up(),
        }
    }

    /// Counts the childs of the path in the AST.
    pub fn count_childs(&self, current_passage: &mut Option<ASTNode>, path: Vec<usize>) -> usize {
        if let Some(_) = path.first() {
            current_passage.as_ref().unwrap().count_childs(path.to_vec())
        } else {
            0
        }
    }

    /// Adds a passage to the path in the AST.
    pub fn add_passage(&mut self, token: Token) -> Option<ASTNode> {
        self.path.clear();
        Some(ASTNode::Passage(NodePassage { category: token, childs: Vec::new() }))
    }

    /// Adds a child to the path in the AST.
    pub fn add_child(&mut self, current_passage_opt: &mut Option<ASTNode>, token: Token) -> Option<ASTNode> {
        if let Some(ref mut current_passage) = current_passage_opt.as_mut() {
            current_passage.add_child(self.path.to_vec(), token);
            None
        } else {
            Some(ASTNode::Default(NodeDefault { category: token, childs: Vec::new() }))
        }
    }

    /// Adds a child and adds the child to the current path.
    pub fn child_down(&mut self, current_passage: &mut Option<ASTNode>, token: Token) -> Option<ASTNode> {
        if token.clone().is_same_token(&TokMacroIf { location: (0, 0) }) ||
           token.clone().is_same_token(&TokMacroElseIf { location: (0, 0) }) {
            self.is_in_if_expression = true;
        }

        let ast_count_childs = current_passage.as_ref().unwrap().count_childs(self.path.to_vec());
        let result = self.add_child(current_passage, token);
        self.path.push(ast_count_childs);
        result
    }

    /// Adds one child and goes down. Adds a second child and goes down.
    pub fn two_childs_down(&mut self, current_passage: &mut Option<ASTNode>, child1: Token, child2: Token) -> Option<ASTNode> {
        match self.child_down(current_passage, child1) {
            Some(new_passage) => {
                self.child_down(&mut Some(new_passage.clone()), child2);
                Some(new_passage)
            },
            None => {
                self.child_down(current_passage, child2)
            }
        }

    }

    /// Goes one level up in the path.
    pub fn up(&mut self) -> Option<ASTNode> {
        self.path.pop();
        None
    }

    /// This goes one level up or goes out of an if-expression if possible.
    pub fn up_special(&mut self) -> Option<ASTNode> {
        if !self.is_in_if_expression {
            self.path.pop();
        } else {
            self.is_in_if_expression = false;
        }
        None
    }

    /// Adds a child and goes one level up.
    pub fn child_up(&mut self, current_passage: &mut Option<ASTNode>, token: Token) -> Option<ASTNode> {
        let result = self.add_child(current_passage, token);
        self.up();
        result
    }

    /// Goes one level up and adds a child.
    pub fn up_child(&mut self, current_passage: &mut Option<ASTNode>, token: Token) -> Option<ASTNode> {
        self.up();
        self.add_child(current_passage, token)
    }

    /// Goes one level up, adds a child and adds the added child to the path.
    pub fn up_child_down(&mut self, current_passage: &mut Option<ASTNode>, token: Token) -> Option<ASTNode> {
        self.up();
        self.child_down(current_passage, token)
    }

    /// Goes two levels up.
    pub fn two_up(&mut self) -> Option<ASTNode> {
        self.up();
        self.up();
        None
    }
}

// ================================
// node types

/// This is a generic node.
///
/// It can either be a standard node or a passage node.
#[derive(Clone)]
pub enum ASTNode {
    /// A standard node
    Default (NodeDefault),

    /// Passage node
    Passage (NodePassage)
}

/// This is a passage node.
///
/// Passage nodes may only appear as root objects, they
/// are, by convention, never children of other nodes.
#[derive(Clone)]
pub struct NodePassage {
    /// The Token of the node
    pub category: Token,

    /// A list of all the childs of the node
    pub childs: Vec<ASTNode>,
    /*tags: Vec<ASTNode>*/
}

/// This is a default node.
#[derive(Clone)]
pub struct NodeDefault {
    /// The Token of the node
    pub category: Token,

    /// A list of all the childs of the node
    pub childs: Vec<ASTNode>
}

/// The implementation of a node.
impl ASTNode {

    /// Adds a child to the path in the AST
    pub fn add_child(&mut self, path: Vec<usize>, token: Token) {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            match self {
                &mut ASTNode::Default(ref mut node) => node.childs[*index].add_child(new_path, token),
                &mut ASTNode::Passage(ref mut node) => node.childs[*index].add_child(new_path, token),
            }
        } else {
            match self {
                &mut ASTNode::Default(ref mut node) => node.childs.push(ASTNode::Default(NodeDefault { category: token, childs: Vec::new() } )),
                &mut ASTNode::Passage(ref mut node) => node.childs.push(ASTNode::Default(NodeDefault { category: token, childs: Vec::new() } )),
            }
        }
    }

    /// Counts the childs of the current path in the AST.
    pub fn count_childs(&self, path: Vec<usize>) -> usize {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            match self {
                &ASTNode::Default(ref node) => node.childs[*index].count_childs(new_path),
                &ASTNode::Passage(ref node) => node.childs[*index].count_childs(new_path),
            }
        } else {
            match self {
                &ASTNode::Default(ref node) => node.childs.len(),
                &ASTNode::Passage(ref node) => node.childs.len(),
            }
        }
    }

    /// Checks the current path if there is the token specified by `token`.
    pub fn is_specific_token(&self, token: Token, path: Vec<usize>) -> bool {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            match self {
                &ASTNode::Default(ref node) => node.childs[*index].is_specific_token(token, new_path),
                &ASTNode::Passage(ref node) => node.childs[*index].is_specific_token(token, new_path),
            }
        } else {
            match self {
                &ASTNode::Default(ref node) => {
                    token == node.category
                },
                &ASTNode::Passage(ref node) => {
                    token == node.category
                },
            }
        }
    }

    /// Returns the current category (Token) of a node.
    pub fn category(&self) -> Token {
        match self {
            &ASTNode::Passage(ref t) => {
                t.category.clone()
            },
            &ASTNode::Default(ref t) => {
                t.category.clone()
            }
        }
    }

    /// Returns all child trees of a node.
    pub fn childs(&self) -> &Vec<ASTNode> {
        match self {
            &ASTNode::Passage(ref t) => {
                &t.childs
            },
            &ASTNode::Default(ref t) => {
                &t.childs
            }
        }
    }

    /// Formats an ASTNode with the specified `indent` for debug printing.
    fn fmt_node(&self, f: &mut Formatter, indent: usize) -> Result {
        let mut spaces = "".to_string();
        for _ in 0..indent {
            spaces.push_str(" ");
        }

        try!(f.write_fmt(format_args!("{}|- : {:?}\n", spaces, self.category())));

        for child in self.childs().iter() {
            try!(child.fmt_node(f, indent+2));
        }
        Ok(())
    }

    /// Unwraps the ASTNode as NodeDefault.
    ///
    /// # Panics
    /// Panics if the ASTNode is not a NodeDefault.
    pub fn as_default(self) -> NodeDefault {
        match self {
            ASTNode::Default(def) => def,
            _ => panic!("Node cannot be unwrapped as NodeDefault!")
        }
    }

    /// Goes through the whole tree and parses the expressions.
    fn parse_expressions(&mut self, cfg: &Config) {
        match self {
            &mut ASTNode::Passage(ref mut node) => {
                for mut child in node.childs.iter_mut() {
                    child.parse_expressions(cfg);
                }
            },
            &mut ASTNode::Default(ref mut node) => {
                match &node.category {
                    &TokExpression => {
                        expressionparser::ExpressionParser::parse(node, cfg);
                    },
                    _ => ()
                }

                for mut child in node.childs.iter_mut() {
                    child.parse_expressions(cfg);
                }
            }
        }
    }
}

impl Debug for ASTNode {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.fmt_node(f, 0)
    }
}

// ================================
// test functions
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use frontend::*;
    use frontend::lexer::Token;
    use frontend::lexer::Token::*;
    use config::Config;

    /// Creates an AST from the input str by lexing and parsing it.
    fn test_ast(input: &str) -> Vec<ASTNode> {
        let cfg = Config::default_config();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(input.to_string().into_bytes());
        let tokens = lexer::lex(cfg.clone(), &mut cursor);
        let parser = parser::Parser::new(cfg.clone());
        let ast_ops = parser.parse(tokens.inspect(|ref token| {
            println!("{:?}", token);
        }));

        let ast = ASTBuilder::build(cfg, ast_ops);
        ast.collect()
    }

    /// Checks if the expected AST matched the provided AST.
    ///
    /// Panics otherwise.
    fn test_expected(expected: Vec<(Vec<usize>, Token)>, ast: Vec<ASTNode>) {
        for item in expected.iter() {
            /// checks in the ast if there is the token "token"
            fn is_specific_token(ast: Vec<ASTNode>, token: Token, path: Vec<usize>) -> bool {
                if let Some(index) = path.first() {
                    let mut new_path: Vec<usize> = path.to_vec();
                    new_path.remove(0);
                    ast[*index].is_specific_token(token, new_path)
                } else {
                    false
                }
            }

            let b = is_specific_token(ast.clone(), item.1.clone(), item.0.to_vec());
            if b == false {
                println!("FAILED WITH TOKEN {:?} at {:?}", item.0, item.1);
            }
            assert!(b);
        }
    }

    #[test]
    fn text_test() {
        let ast = test_ast("::Start\nTestText\nTestNextLine\n::NextPassage\nOtherText");

        let expected = vec!(
            (vec![0]  , TokPassage {location: (1, 3), name: "Start".to_string()}),
            (vec![0,0], TokText {location: (2, 1), text: "TestText".to_string()}),
            (vec![0,1], TokNewLine {location: (2, 9)} ),
            (vec![0,2], TokText {location: (3, 1), text: "TestNextLine".to_string()}),
            (vec![0,3], TokNewLine {location: (3, 13)}),
            (vec![1]  , TokPassage {location: (4, 3), name: "NextPassage".to_string()}),
            (vec![1,0], TokText {location: (5, 1), text: "OtherText".to_string()}),

        );

        test_expected(expected, ast);
    }

    #[test]
    fn num_expressions_test() {
        let ast = test_ast("::Start\n<<print -12345>>\n<<print 5>>\n<<print 32767>>\n<<print 1*2*3*4*5*6*7>>\n<<print 1*2+3*4+5*6+7>>\n<<print 1*2-3*4-5*6-7>>\n<<print 256/8/4/8>>\n<<print 6300/5/7/9/10>>\n<<print 6300/5/7/-9/10>>\n<<print 1-3>>\n<<print -2+2>>\n<<print (1+2)*(3--4)>>\n<<print (1+2)*(3+4)*(5+6)*(7+8)>>\n<<print (1-2)*(3-4)*(5-6)*(7-8)>>\n<<print ((1-2)*(3+4))*(5-6)*(7-8)>>\n<<print (2*9)/(-7)>>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (1, 3), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (2, 3) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokUnaryMinus { location: (2, 9) }),
            (vec![0,0,0,0,0]          , TokInt { location: (2, 10), value: 12345 }),
            (vec![0,1]                , TokNewLine { location: (2, 17) }),
            (vec![0,2]                , TokMacroPrint { location: (3, 3) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokInt { location: (3, 9), value: 5 }),
            (vec![0,3]                , TokNewLine { location: (3, 12) }),
            (vec![0,4]                , TokMacroPrint { location: (4, 3) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokInt { location: (4, 9), value: 32767 }),
            (vec![0,5]                , TokNewLine { location: (4, 16) }),
            (vec![0,6]                , TokMacroPrint { location: (5, 3) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokNumOp { location: (5, 20), op_name: "*".to_string() }),
            (vec![0,6,0,0,0]          , TokNumOp { location: (5, 18), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0]        , TokNumOp { location: (5, 16), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0,0]      , TokNumOp { location: (5, 14), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0,0,0]    , TokNumOp { location: (5, 12), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0,0,0,0]  , TokNumOp { location: (5, 10), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0,0,0,0,0], TokInt { location: (5, 9), value: 1 }),
            (vec![0,6,0,0,0,0,0,0,0,1], TokInt { location: (5, 11), value: 2 }),
            (vec![0,6,0,0,0,0,0,0,1]  , TokInt { location: (5, 13), value: 3 }),
            (vec![0,6,0,0,0,0,0,1]    , TokInt { location: (5, 15), value: 4 }),
            (vec![0,6,0,0,0,0,1]      , TokInt { location: (5, 17), value: 5 }),
            (vec![0,6,0,0,0,1]        , TokInt { location: (5, 19), value: 6 }),
            (vec![0,6,0,0,1]          , TokInt { location: (5, 21), value: 7 }),
            (vec![0,7]                , TokNewLine { location: (5, 24) }),
            (vec![0,8]                , TokMacroPrint { location: (6, 3) }),
            (vec![0,8,0]              , TokExpression),
            (vec![0,8,0,0]            , TokNumOp { location: (6, 20), op_name: "+".to_string() }),
            (vec![0,8,0,0,0]          , TokNumOp { location: (6, 16), op_name: "+".to_string() }),
            (vec![0,8,0,0,0,0]        , TokNumOp { location: (6, 12), op_name: "+".to_string() }),
            (vec![0,8,0,0,0,0,0]      , TokNumOp { location: (6, 10), op_name: "*".to_string() }),
            (vec![0,8,0,0,0,0,0,0]    , TokInt { location: (6, 9), value: 1 }),
            (vec![0,8,0,0,0,0,0,1]    , TokInt { location: (6, 11), value: 2 }),
            (vec![0,8,0,0,0,0,1]      , TokNumOp { location: (6, 14), op_name: "*".to_string() }),
            (vec![0,8,0,0,0,0,1,0]    , TokInt { location: (6, 13), value: 3 }),
            (vec![0,8,0,0,0,0,1,1]    , TokInt { location: (6, 15), value: 4 }),
            (vec![0,8,0,0,0,1]        , TokNumOp { location: (6, 18), op_name: "*".to_string() }),
            (vec![0,8,0,0,0,1,0]      , TokInt { location: (6, 17), value: 5 }),
            (vec![0,8,0,0,0,1,1]      , TokInt { location: (6, 19), value: 6 }),
            (vec![0,8,0,0,1]          , TokInt { location: (6, 21), value: 7 }),
            (vec![0,9]                , TokNewLine { location: (6, 24) }),
            (vec![0,10]               , TokMacroPrint { location: (7, 3) }),
            (vec![0,10,0]             , TokExpression),
            (vec![0,10,0,0]           , TokNumOp { location: (7, 20), op_name: "-".to_string() }),
            (vec![0,10,0,0,0]         , TokNumOp { location: (7, 16), op_name: "-".to_string() }),
            (vec![0,10,0,0,0,0]       , TokNumOp { location: (7, 12), op_name: "-".to_string() }),
            (vec![0,10,0,0,0,0,0]     , TokNumOp { location: (7, 10), op_name: "*".to_string() }),
            (vec![0,10,0,0,0,0,0,0]   , TokInt { location: (7, 9), value: 1 }),
            (vec![0,10,0,0,0,0,0,1]   , TokInt { location: (7, 11), value: 2 }),
            (vec![0,10,0,0,0,0,1]     , TokNumOp { location: (7, 14), op_name: "*".to_string() }),
            (vec![0,10,0,0,0,0,1,0]   , TokInt { location: (7, 13), value: 3 }),
            (vec![0,10,0,0,0,0,1,1]   , TokInt { location: (7, 15), value: 4 }),
            (vec![0,10,0,0,0,1]       , TokNumOp { location: (7, 18), op_name: "*".to_string() }),
            (vec![0,10,0,0,0,1,0]     , TokInt { location: (7, 17), value: 5 }),
            (vec![0,10,0,0,0,1,1]     , TokInt { location: (7, 19), value: 6 }),
            (vec![0,10,0,0,1]         , TokInt { location: (7, 21), value: 7 }),
            (vec![0,11]               , TokNewLine { location: (7, 24) }),
            (vec![0,12]               , TokMacroPrint { location: (8, 3) }),
            (vec![0,12,0]             , TokExpression),
            (vec![0,12,0,0]           , TokNumOp { location: (8, 16), op_name: "/".to_string() }),
            (vec![0,12,0,0,0]         , TokNumOp { location: (8, 14), op_name: "/".to_string() }),
            (vec![0,12,0,0,0,0]       , TokNumOp { location: (8, 12), op_name: "/".to_string() }),
            (vec![0,12,0,0,0,0,0]     , TokInt { location: (8, 9), value: 256 }),
            (vec![0,12,0,0,0,0,1]     , TokInt { location: (8, 13), value: 8 }),
            (vec![0,12,0,0,0,1]       , TokInt { location: (8, 15), value: 4 }),
            (vec![0,12,0,0,1]         , TokInt { location: (8, 17), value: 8 }),
            (vec![0,13]               , TokNewLine { location: (8, 20) }),
            (vec![0,14]               , TokMacroPrint { location: (9, 3) }),
            (vec![0,14,0]             , TokExpression),
            (vec![0,14,0,0]           , TokNumOp { location: (9, 19), op_name: "/".to_string() }),
            (vec![0,14,0,0,0]         , TokNumOp { location: (9, 17), op_name: "/".to_string() }),
            (vec![0,14,0,0,0,0]       , TokNumOp { location: (9, 15), op_name: "/".to_string() }),
            (vec![0,14,0,0,0,0,0]     , TokNumOp { location: (9, 13), op_name: "/".to_string() }),
            (vec![0,14,0,0,0,0,0,0]   , TokInt { location: (9, 9), value: 6300 }),
            (vec![0,14,0,0,0,0,0,1]   , TokInt { location: (9, 14), value: 5 }),
            (vec![0,14,0,0,0,0,1]     , TokInt { location: (9, 16), value: 7 }),
            (vec![0,14,0,0,0,1]       , TokInt { location: (9, 18), value: 9 }),
            (vec![0,14,0,0,1]         , TokInt { location: (9, 20), value: 10 }),
            (vec![0,15]               , TokNewLine { location: (9, 24) }),
            (vec![0,16]               , TokMacroPrint { location: (10, 3) }),
            (vec![0,16,0]             , TokExpression),
            (vec![0,16,0,0]           , TokNumOp { location: (10, 20), op_name: "/".to_string() }),
            (vec![0,16,0,0,0]         , TokNumOp { location: (10, 17), op_name: "/".to_string() }),
            (vec![0,16,0,0,0,0]       , TokNumOp { location: (10, 15), op_name: "/".to_string() }),
            (vec![0,16,0,0,0,0,0]     , TokNumOp { location: (10, 13), op_name: "/".to_string() }),
            (vec![0,16,0,0,0,0,0,0]   , TokInt { location: (10, 9), value: 6300 }),
            (vec![0,16,0,0,0,0,0,1]   , TokInt { location: (10, 14), value: 5 }),
            (vec![0,16,0,0,0,0,1]     , TokInt { location: (10, 16), value: 7 }),
            (vec![0,16,0,0,0,1]       , TokUnaryMinus { location: (10, 18) }),
            (vec![0,16,0,0,0,1,0]     , TokInt { location: (10, 19), value: 9 }),
            (vec![0,16,0,0,1]         , TokInt { location: (10, 21), value: 10 }),
            (vec![0,17]               , TokNewLine { location: (10, 25) }),
            (vec![0,18]               , TokMacroPrint { location: (11, 3) }),
            (vec![0,18,0]             , TokExpression),
            (vec![0,18,0,0]           , TokNumOp { location: (11, 10), op_name: "-".to_string() }),
            (vec![0,18,0,0,0]         , TokInt { location: (11, 9), value: 1 }),
            (vec![0,18,0,0,1]         , TokInt { location: (11, 11), value: 3 }),
            (vec![0,19]               , TokNewLine { location: (11, 14) }),
            (vec![0,20]               , TokMacroPrint { location: (12, 3) }),
            (vec![0,20,0]             , TokExpression),
            (vec![0,20,0,0]           , TokNumOp { location: (12, 11), op_name: "+".to_string() }),
            (vec![0,20,0,0,0]         , TokUnaryMinus { location: (12, 9) }),
            (vec![0,20,0,0,0,0]       , TokInt { location: (12, 10), value: 2 }),
            (vec![0,20,0,0,1]         , TokInt { location: (12, 12), value: 2 }),
            (vec![0,21]               , TokNewLine { location: (12, 15) }),
            (vec![0,22]               , TokMacroPrint { location: (13, 3) }),
            (vec![0,22,0]             , TokExpression),
            (vec![0,22,0,0]           , TokNumOp { location: (13, 14), op_name: "*".to_string() }),
            (vec![0,22,0,0,0]         , TokNumOp { location: (13, 11), op_name: "+".to_string() }),
            (vec![0,22,0,0,0,0]       , TokInt { location: (13, 10), value: 1 }),
            (vec![0,22,0,0,0,1]       , TokInt { location: (13, 12), value: 2 }),
            (vec![0,22,0,0,1]         , TokNumOp { location: (13, 17), op_name: "-".to_string() }),
            (vec![0,22,0,0,1,0]       , TokInt { location: (13, 16), value: 3 }),
            (vec![0,22,0,0,1,1]       , TokUnaryMinus { location: (13, 18) }),
            (vec![0,22,0,0,1,1,0]     , TokInt { location: (13, 19), value: 4 }),
            (vec![0,23]               , TokNewLine { location: (13, 23) }),
            (vec![0,24]               , TokMacroPrint { location: (14, 3) }),
            (vec![0,24,0]             , TokExpression),
            (vec![0,24,0,0]           , TokNumOp { location: (14, 26), op_name: "*".to_string() }),
            (vec![0,24,0,0,0]         , TokNumOp { location: (14, 20), op_name: "*".to_string() }),
            (vec![0,24,0,0,0,0]       , TokNumOp { location: (14, 14), op_name: "*".to_string() }),
            (vec![0,24,0,0,0,0,0]     , TokNumOp { location: (14, 11), op_name: "+".to_string() }),
            (vec![0,24,0,0,0,0,0,0]   , TokInt { location: (14, 10), value: 1 }),
            (vec![0,24,0,0,0,0,0,1]   , TokInt { location: (14, 12), value: 2 }),
            (vec![0,24,0,0,0,0,1]     , TokNumOp { location: (14, 17), op_name: "+".to_string() }),
            (vec![0,24,0,0,0,0,1,0]   , TokInt { location: (14, 16), value: 3 }),
            (vec![0,24,0,0,0,0,1,1]   , TokInt { location: (14, 18), value: 4 }),
            (vec![0,24,0,0,0,1]       , TokNumOp { location: (14, 23), op_name: "+".to_string() }),
            (vec![0,24,0,0,0,1,0]     , TokInt { location: (14, 22), value: 5 }),
            (vec![0,24,0,0,0,1,1]     , TokInt { location: (14, 24), value: 6 }),
            (vec![0,24,0,0,1]         , TokNumOp { location: (14, 29), op_name: "+".to_string() }),
            (vec![0,24,0,0,1,0]       , TokInt { location: (14, 28), value: 7 }),
            (vec![0,24,0,0,1,1]       , TokInt { location: (14, 30), value: 8 }),
            (vec![0,25]               , TokNewLine { location: (14, 34) }),
            (vec![0,26]               , TokMacroPrint { location: (15, 3) }),
            (vec![0,26,0]             , TokExpression),
            (vec![0,26,0,0]           , TokNumOp { location: (15, 26), op_name: "*".to_string() }),
            (vec![0,26,0,0,0]         , TokNumOp { location: (15, 20), op_name: "*".to_string() }),
            (vec![0,26,0,0,0,0]       , TokNumOp { location: (15, 14), op_name: "*".to_string() }),
            (vec![0,26,0,0,0,0,0]     , TokNumOp { location: (15, 11), op_name: "-".to_string() }),
            (vec![0,26,0,0,0,0,0,0]   , TokInt { location: (15, 10), value: 1 }),
            (vec![0,26,0,0,0,0,0,1]   , TokInt { location: (15, 12), value: 2 }),
            (vec![0,26,0,0,0,0,1]     , TokNumOp { location: (15, 17), op_name: "-".to_string() }),
            (vec![0,26,0,0,0,0,1,0]   , TokInt { location: (15, 16), value: 3 }),
            (vec![0,26,0,0,0,0,1,1]   , TokInt { location: (15, 18), value: 4 }),
            (vec![0,26,0,0,0,1]       , TokNumOp { location: (15, 23), op_name: "-".to_string() }),
            (vec![0,26,0,0,0,1,0]     , TokInt { location: (15, 22), value: 5 }),
            (vec![0,26,0,0,0,1,1]     , TokInt { location: (15, 24), value: 6 }),
            (vec![0,26,0,0,1]         , TokNumOp { location: (15, 29), op_name: "-".to_string() }),
            (vec![0,26,0,0,1,0]       , TokInt { location: (15, 28), value: 7 }),
            (vec![0,26,0,0,1,1]       , TokInt { location: (15, 30), value: 8 }),
            (vec![0,27]               , TokNewLine { location: (15, 34) }),
            (vec![0,28]               , TokMacroPrint { location: (16, 3) }),
            (vec![0,28,0]             , TokExpression),
            (vec![0,28,0,0]           , TokNumOp { location: (16, 28), op_name: "*".to_string() }),
            (vec![0,28,0,0,0]         , TokNumOp { location: (16, 22), op_name: "*".to_string() }),
            (vec![0,28,0,0,0,0]       , TokNumOp { location: (16, 15), op_name: "*".to_string() }),
            (vec![0,28,0,0,0,0,0]     , TokNumOp { location: (16, 12), op_name: "-".to_string() }),
            (vec![0,28,0,0,0,0,0,0]   , TokInt { location: (16, 11), value: 1 }),
            (vec![0,28,0,0,0,0,0,1]   , TokInt { location: (16, 13), value: 2 }),
            (vec![0,28,0,0,0,0,1]     , TokNumOp { location: (16, 18), op_name: "+".to_string() }),
            (vec![0,28,0,0,0,0,1,0]   , TokInt { location: (16, 17), value: 3 }),
            (vec![0,28,0,0,0,0,1,1]   , TokInt { location: (16, 19), value: 4 }),
            (vec![0,28,0,0,0,1]       , TokNumOp { location: (16, 25), op_name: "-".to_string() }),
            (vec![0,28,0,0,0,1,0]     , TokInt { location: (16, 24), value: 5 }),
            (vec![0,28,0,0,0,1,1]     , TokInt { location: (16, 26), value: 6 }),
            (vec![0,28,0,0,1]         , TokNumOp { location: (16, 31), op_name: "-".to_string() }),
            (vec![0,28,0,0,1,0]       , TokInt { location: (16, 30), value: 7 }),
            (vec![0,28,0,0,1,1]       , TokInt { location: (16, 32), value: 8 }),
            (vec![0,29]               , TokNewLine { location: (16, 36) }),
            (vec![0,30]               , TokMacroPrint { location: (17, 3) }),
            (vec![0,30,0]             , TokExpression),
            (vec![0,30,0,0]           , TokNumOp { location: (17, 14), op_name: "/".to_string() }),
            (vec![0,30,0,0,0]         , TokNumOp { location: (17, 11), op_name: "*".to_string() }),
            (vec![0,30,0,0,0,0]       , TokInt { location: (17, 10), value: 2 }),
            (vec![0,30,0,0,0,1]       , TokInt { location: (17, 12), value: 9 }),
            (vec![0,30,0,0,1]         , TokUnaryMinus { location: (17, 16) }),
            (vec![0,30,0,0,1,0]       , TokInt { location: (17, 17), value: 7 }),
            (vec![0,31]               , TokNewLine { location: (17, 21) }),
        );

        test_expected(expected, ast);
    }

    #[test]
    fn log_expressions_test() {
        let ast = test_ast("::Start\n<<print false>>\n<<print true>>\n<<print not false>>\n<<print not true>>\n<<print not-5>>\n<<print not5>>\n<<print not0>>\n<<print true and true>>\n<<print true and false>>\n<<print false and true>>\n<<print false and false>>\n<<print true or true>>\n<<print true or false>>\n<<print false or true>>\n<<print false or false>>\n<<print false or true and true>>\n<<print false or true or false>>\n<<print true or false and true and false or true>>\n<<print (true or false) and false>>\n<<print (true or false) and (true or true)>>\n<<print (true and true)>>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (1, 3), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (2, 3) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokBoolean { location: (2, 9), value: "false".to_string() }),
            (vec![0,1]                , TokNewLine { location: (2, 16) }),
            (vec![0,2]                , TokMacroPrint { location: (3, 3) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokBoolean { location: (3, 9), value: "true".to_string() }),
            (vec![0,3]                , TokNewLine { location: (3, 15) }),
            (vec![0,4]                , TokMacroPrint { location: (4, 3) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokLogOp { location: (4, 9), op_name: "not".to_string() }),
            (vec![0,4,0,0,0]          , TokBoolean { location: (4, 13), value: "false".to_string() }),
            (vec![0,5]                , TokNewLine { location: (4, 20) }),
            (vec![0,6]                , TokMacroPrint { location: (5, 3) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokLogOp { location: (5, 9), op_name: "not".to_string() }),
            (vec![0,6,0,0,0]          , TokBoolean { location: (5, 13), value: "true".to_string() }),
            (vec![0,7]                , TokNewLine { location: (5, 19) }),
            (vec![0,8]                , TokMacroPrint { location: (6, 3) }),
            (vec![0,8,0]              , TokExpression),
            (vec![0,8,0,0]            , TokLogOp { location: (6, 9), op_name: "not".to_string() }),
            (vec![0,8,0,0,0]          , TokUnaryMinus { location: (6, 12) }),
            (vec![0,8,0,0,0,0]        , TokInt { location: (6, 13), value: 5 }),
            (vec![0,9]                , TokNewLine { location: (6, 16) }),
            (vec![0,10]               , TokMacroPrint { location: (7, 3) }),
            (vec![0,10,0]             , TokExpression),
            (vec![0,10,0,0]           , TokLogOp { location: (7, 9), op_name: "not".to_string() }),
            (vec![0,10,0,0,0]         , TokInt { location: (7, 12), value: 5 }),
            (vec![0,11]               , TokNewLine { location: (7, 15) }),
            (vec![0,12]               , TokMacroPrint { location: (8, 3) }),
            (vec![0,12,0]             , TokExpression),
            (vec![0,12,0,0]           , TokLogOp { location: (8, 9), op_name: "not".to_string() }),
            (vec![0,12,0,0,0]         , TokInt { location: (8, 12), value: 0 }),
            (vec![0,13]               , TokNewLine { location: (8, 15) }),
            (vec![0,14]               , TokMacroPrint { location: (9, 3) }),
            (vec![0,14,0]             , TokExpression),
            (vec![0,14,0,0]           , TokLogOp { location: (9, 14), op_name: "and".to_string() }),
            (vec![0,14,0,0,0]         , TokBoolean { location: (9, 9), value: "true".to_string() }),
            (vec![0,14,0,0,1]         , TokBoolean { location: (9, 18), value: "true".to_string() }),
            (vec![0,15]               , TokNewLine { location: (9, 24) }),
            (vec![0,16]               , TokMacroPrint { location: (10, 3) }),
            (vec![0,16,0]             , TokExpression),
            (vec![0,16,0,0]           , TokLogOp { location: (10, 14), op_name: "and".to_string() }),
            (vec![0,16,0,0,0]         , TokBoolean { location: (10, 9), value: "true".to_string() }),
            (vec![0,16,0,0,1]         , TokBoolean { location: (10, 18), value: "false".to_string() }),
            (vec![0,17]               , TokNewLine { location: (10, 25) }),
            (vec![0,18]               , TokMacroPrint { location: (11, 3) }),
            (vec![0,18,0]             , TokExpression),
            (vec![0,18,0,0]           , TokLogOp { location: (11, 15), op_name: "and".to_string() }),
            (vec![0,18,0,0,0]         , TokBoolean { location: (11, 9), value: "false".to_string() }),
            (vec![0,18,0,0,1]         , TokBoolean { location: (11, 19), value: "true".to_string() }),
            (vec![0,19]               , TokNewLine { location: (11, 25) }),
            (vec![0,20]               , TokMacroPrint { location: (12, 3) }),
            (vec![0,20,0]             , TokExpression),
            (vec![0,20,0,0]           , TokLogOp { location: (12, 15), op_name: "and".to_string() }),
            (vec![0,20,0,0,0]         , TokBoolean { location: (12, 9), value: "false".to_string() }),
            (vec![0,20,0,0,1]         , TokBoolean { location: (12, 19), value: "false".to_string() }),
            (vec![0,21]               , TokNewLine { location: (12, 26) }),
            (vec![0,22]               , TokMacroPrint { location: (13, 3) }),
            (vec![0,22,0]             , TokExpression),
            (vec![0,22,0,0]           , TokLogOp { location: (13, 14), op_name: "or".to_string() }),
            (vec![0,22,0,0,0]         , TokBoolean { location: (13, 9), value: "true".to_string() }),
            (vec![0,22,0,0,1]         , TokBoolean { location: (13, 17), value: "true".to_string() }),
            (vec![0,23]               , TokNewLine { location: (13, 23) }),
            (vec![0,24]               , TokMacroPrint { location: (14, 3) }),
            (vec![0,24,0]             , TokExpression),
            (vec![0,24,0,0]           , TokLogOp { location: (14, 14), op_name: "or".to_string() }),
            (vec![0,24,0,0,0]         , TokBoolean { location: (14, 9), value: "true".to_string() }),
            (vec![0,24,0,0,1]         , TokBoolean { location: (14, 17), value: "false".to_string() }),
            (vec![0,25]               , TokNewLine { location: (14, 24) }),
            (vec![0,26]               , TokMacroPrint { location: (15, 3) }),
            (vec![0,26,0]             , TokExpression),
            (vec![0,26,0,0]           , TokLogOp { location: (15, 15), op_name: "or".to_string() }),
            (vec![0,26,0,0,0]         , TokBoolean { location: (15, 9), value: "false".to_string() }),
            (vec![0,26,0,0,1]         , TokBoolean { location: (15, 18), value: "true".to_string() }),
            (vec![0,27]               , TokNewLine { location: (15, 24) }),
            (vec![0,28]               , TokMacroPrint { location: (16, 3) }),
            (vec![0,28,0]             , TokExpression),
            (vec![0,28,0,0]           , TokLogOp { location: (16, 15), op_name: "or".to_string() }),
            (vec![0,28,0,0,0]         , TokBoolean { location: (16, 9), value: "false".to_string() }),
            (vec![0,28,0,0,1]         , TokBoolean { location: (16, 18), value: "false".to_string() }),
            (vec![0,29]               , TokNewLine { location: (16, 25) }),
            (vec![0,30]               , TokMacroPrint { location: (17, 3) }),
            (vec![0,30,0]             , TokExpression),
            (vec![0,30,0,0]           , TokLogOp { location: (17, 15), op_name: "or".to_string() }),
            (vec![0,30,0,0,0]         , TokBoolean { location: (17, 9), value: "false".to_string() }),
            (vec![0,30,0,0,1]         , TokLogOp { location: (17, 23), op_name: "and".to_string() }),
            (vec![0,30,0,0,1,0]       , TokBoolean { location: (17, 18), value: "true".to_string() }),
            (vec![0,30,0,0,1,1]       , TokBoolean { location: (17, 27), value: "true".to_string() }),
            (vec![0,31]               , TokNewLine { location: (17, 33) }),
            (vec![0,32]               , TokMacroPrint { location: (18, 3) }),
            (vec![0,32,0]             , TokExpression),
            (vec![0,32,0,0]           , TokLogOp { location: (18, 23), op_name: "or".to_string() }),
            (vec![0,32,0,0,0]         , TokLogOp { location: (18, 15), op_name: "or".to_string() }),
            (vec![0,32,0,0,0,0]       , TokBoolean { location: (18, 9), value: "false".to_string() }),
            (vec![0,32,0,0,0,1]       , TokBoolean { location: (18, 18), value: "true".to_string() }),
            (vec![0,32,0,0,1]         , TokBoolean { location: (18, 26), value: "false".to_string() }),
            (vec![0,33]               , TokNewLine { location: (18, 33) }),
            (vec![0,34]               , TokMacroPrint { location: (19, 3) }),
            (vec![0,34,0]             , TokExpression),
            (vec![0,34,0,0]           , TokLogOp { location: (19, 42), op_name: "or".to_string() }),
            (vec![0,34,0,0,0]         , TokLogOp { location: (19, 14), op_name: "or".to_string() }),
            (vec![0,34,0,0,0,0]       , TokBoolean { location: (19, 9), value: "true".to_string() }),
            (vec![0,34,0,0,0,1]       , TokLogOp { location: (19, 32), op_name: "and".to_string() }),
            (vec![0,34,0,0,0,1,0]     , TokLogOp { location: (19, 23), op_name: "and".to_string() }),
            (vec![0,34,0,0,0,1,0,0]   , TokBoolean { location: (19, 17), value: "false".to_string() }),
            (vec![0,34,0,0,0,1,0,1]   , TokBoolean { location: (19, 27), value: "true".to_string() }),
            (vec![0,34,0,0,0,1,1]     , TokBoolean { location: (19, 36), value: "false".to_string() }),
            (vec![0,34,0,0,1]         , TokBoolean { location: (19, 45), value: "true".to_string() }),
            (vec![0,35]               , TokNewLine { location: (19, 51) }),
            (vec![0,36]               , TokMacroPrint { location: (20, 3) }),
            (vec![0,36,0]             , TokExpression),
            (vec![0,36,0,0]           , TokLogOp { location: (20, 25), op_name: "and".to_string() }),
            (vec![0,36,0,0,0]         , TokLogOp { location: (20, 15), op_name: "or".to_string() }),
            (vec![0,36,0,0,0,0]       , TokBoolean { location: (20, 10), value: "true".to_string() }),
            (vec![0,36,0,0,0,1]       , TokBoolean { location: (20, 18), value: "false".to_string() }),
            (vec![0,36,0,0,1]         , TokBoolean { location: (20, 29), value: "false".to_string() }),
            (vec![0,37]               , TokNewLine { location: (20, 36) }),
            (vec![0,38]               , TokMacroPrint { location: (21, 3) }),
            (vec![0,38,0]             , TokExpression),
            (vec![0,38,0,0]           , TokLogOp { location: (21, 25), op_name: "and".to_string() }),
            (vec![0,38,0,0,0]         , TokLogOp { location: (21, 15), op_name: "or".to_string() }),
            (vec![0,38,0,0,0,0]       , TokBoolean { location: (21, 10), value: "true".to_string() }),
            (vec![0,38,0,0,0,1]       , TokBoolean { location: (21, 18), value: "false".to_string() }),
            (vec![0,38,0,0,1]         , TokLogOp { location: (21, 35), op_name: "or".to_string() }),
            (vec![0,38,0,0,1,0]       , TokBoolean { location: (21, 30), value: "true".to_string() }),
            (vec![0,38,0,0,1,1]       , TokBoolean { location: (21, 38), value: "true".to_string() }),
            (vec![0,39]               , TokNewLine { location: (21, 45) }),
            (vec![0,40]               , TokMacroPrint { location: (22, 3) }),
            (vec![0,40,0]             , TokExpression),
            (vec![0,40,0,0]           , TokLogOp { location: (22, 15), op_name: "and".to_string() }),
            (vec![0,40,0,0,0]         , TokBoolean { location: (22, 10), value: "true".to_string() }),
            (vec![0,40,0,0,1]         , TokBoolean { location: (22, 19), value: "true".to_string() }),
            (vec![0,41]               , TokNewLine { location: (22, 26) }),
        );

        test_expected(expected, ast);
    }

    #[test]
    fn comp_expressions_test() {
        let ast = test_ast("::Start\n<<print 5==5>>\n<<print 12345==1234>>\n<<print 5is5>>\n<<print 12345is1234>>\n<<print 10>=10>>\n<<print 15>=10>>\n<<print 8>=10>>\n<<print 10gte10>>\n<<print 15gte10>>\n<<print 8gte10>>\n<<print 10<=10>>\n<<print 15<=10>>\n<<print 8<=10>>\n<<print 10lte10>>\n<<print 8lte10>>\n<<print 15<10>>\n<<print 8<10>>\n<<print 10lt10>>\n<<print 15lt10>>\n<<print 8lt10>>\n<<print 10>10>>\n<<print 15>10>>\n<<print 8>10>>\n<<print 10gt10>>\n<<print 15gt10>>\n<<print 8gt10>>\n<<print 5neq5>>\n<<print 12345neq1234>>\n<<print 5!=5>>\n<<print 12345 != 1234>>\n<<print \"hallo\"==\"hallo\">>\n<<print \"hallo\"==\"hallo2\">>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (1, 3), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (2, 3) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokCompOp { location: (2, 10), op_name: "==".to_string() }),
            (vec![0,0,0,0,0]          , TokInt { location: (2, 9), value: 5 }),
            (vec![0,0,0,0,1]          , TokInt { location: (2, 12), value: 5 }),
            (vec![0,1]                , TokNewLine { location: (2, 15) }),
            (vec![0,2]                , TokMacroPrint { location: (3, 3) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokCompOp { location: (3, 14), op_name: "==".to_string() }),
            (vec![0,2,0,0,0]          , TokInt { location: (3, 9), value: 12345 }),
            (vec![0,2,0,0,1]          , TokInt { location: (3, 16), value: 1234 }),
            (vec![0,3]                , TokNewLine { location: (3, 22) }),
            (vec![0,4]                , TokMacroPrint { location: (4, 3) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokCompOp { location: (4, 10), op_name: "is".to_string() }),
            (vec![0,4,0,0,0]          , TokInt { location: (4, 9), value: 5 }),
            (vec![0,4,0,0,1]          , TokInt { location: (4, 12), value: 5 }),
            (vec![0,5]                , TokNewLine { location: (4, 15) }),
            (vec![0,6]                , TokMacroPrint { location: (5, 3) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokCompOp { location: (5, 14), op_name: "is".to_string() }),
            (vec![0,6,0,0,0]          , TokInt { location: (5, 9), value: 12345 }),
            (vec![0,6,0,0,1]          , TokInt { location: (5, 16), value: 1234 }),
            (vec![0,7]                , TokNewLine { location: (5, 22) }),
            (vec![0,8]                , TokMacroPrint { location: (6, 3) }),
            (vec![0,8,0]              , TokExpression),
            (vec![0,8,0,0]            , TokCompOp { location: (6, 11), op_name: ">=".to_string() }),
            (vec![0,8,0,0,0]          , TokInt { location: (6, 9), value: 10 }),
            (vec![0,8,0,0,1]          , TokInt { location: (6, 13), value: 10 }),
            (vec![0,9]                , TokNewLine { location: (6, 17) }),
            (vec![0,10]               , TokMacroPrint { location: (7, 3) }),
            (vec![0,10,0]             , TokExpression),
            (vec![0,10,0,0]           , TokCompOp { location: (7, 11), op_name: ">=".to_string() }),
            (vec![0,10,0,0,0]         , TokInt { location: (7, 9), value: 15 }),
            (vec![0,10,0,0,1]         , TokInt { location: (7, 13), value: 10 }),
            (vec![0,11]               , TokNewLine { location: (7, 17) }),
            (vec![0,12]               , TokMacroPrint { location: (8, 3) }),
            (vec![0,12,0]             , TokExpression),
            (vec![0,12,0,0]           , TokCompOp { location: (8, 10), op_name: ">=".to_string() }),
            (vec![0,12,0,0,0]         , TokInt { location: (8, 9), value: 8 }),
            (vec![0,12,0,0,1]         , TokInt { location: (8, 12), value: 10 }),
            (vec![0,13]               , TokNewLine { location: (8, 16) }),
            (vec![0,14]               , TokMacroPrint { location: (9, 3) }),
            (vec![0,14,0]             , TokExpression),
            (vec![0,14,0,0]           , TokCompOp { location: (9, 11), op_name: "gte".to_string() }),
            (vec![0,14,0,0,0]         , TokInt { location: (9, 9), value: 10 }),
            (vec![0,14,0,0,1]         , TokInt { location: (9, 14), value: 10 }),
            (vec![0,15]               , TokNewLine { location: (9, 18) }),
            (vec![0,16]               , TokMacroPrint { location: (10, 3) }),
            (vec![0,16,0]             , TokExpression),
            (vec![0,16,0,0]           , TokCompOp { location: (10, 11), op_name: "gte".to_string() }),
            (vec![0,16,0,0,0]         , TokInt { location: (10, 9), value: 15 }),
            (vec![0,16,0,0,1]         , TokInt { location: (10, 14), value: 10 }),
            (vec![0,17]               , TokNewLine { location: (10, 18) }),
            (vec![0,18]               , TokMacroPrint { location: (11, 3) }),
            (vec![0,18,0]             , TokExpression),
            (vec![0,18,0,0]           , TokCompOp { location: (11, 10), op_name: "gte".to_string() }),
            (vec![0,18,0,0,0]         , TokInt { location: (11, 9), value: 8 }),
            (vec![0,18,0,0,1]         , TokInt { location: (11, 13), value: 10 }),
            (vec![0,19]               , TokNewLine { location: (11, 17) }),
            (vec![0,20]               , TokMacroPrint { location: (12, 3) }),
            (vec![0,20,0]             , TokExpression),
            (vec![0,20,0,0]           , TokCompOp { location: (12, 11), op_name: "<=".to_string() }),
            (vec![0,20,0,0,0]         , TokInt { location: (12, 9), value: 10 }),
            (vec![0,20,0,0,1]         , TokInt { location: (12, 13), value: 10 }),
            (vec![0,21]               , TokNewLine { location: (12, 17) }),
            (vec![0,22]               , TokMacroPrint { location: (13, 3) }),
            (vec![0,22,0]             , TokExpression),
            (vec![0,22,0,0]           , TokCompOp { location: (13, 11), op_name: "<=".to_string() }),
            (vec![0,22,0,0,0]         , TokInt { location: (13, 9), value: 15 }),
            (vec![0,22,0,0,1]         , TokInt { location: (13, 13), value: 10 }),
            (vec![0,23]               , TokNewLine { location: (13, 17) }),
            (vec![0,24]               , TokMacroPrint { location: (14, 3) }),
            (vec![0,24,0]             , TokExpression),
            (vec![0,24,0,0]           , TokCompOp { location: (14, 10), op_name: "<=".to_string() }),
            (vec![0,24,0,0,0]         , TokInt { location: (14, 9), value: 8 }),
            (vec![0,24,0,0,1]         , TokInt { location: (14, 12), value: 10 }),
            (vec![0,25]               , TokNewLine { location: (14, 16) }),
            (vec![0,26]               , TokMacroPrint { location: (15, 3) }),
            (vec![0,26,0]             , TokExpression),
            (vec![0,26,0,0]           , TokCompOp { location: (15, 11), op_name: "lte".to_string() }),
            (vec![0,26,0,0,0]         , TokInt { location: (15, 9), value: 10 }),
            (vec![0,26,0,0,1]         , TokInt { location: (15, 14), value: 10 }),
            (vec![0,27]               , TokNewLine { location: (15, 18) }),
            (vec![0,28]               , TokMacroPrint { location: (16, 3) }),
            (vec![0,28,0]             , TokExpression),
            (vec![0,28,0,0]           , TokCompOp { location: (16, 10), op_name: "lte".to_string() }),
            (vec![0,28,0,0,0]         , TokInt { location: (16, 9), value: 8 }),
            (vec![0,28,0,0,1]         , TokInt { location: (16, 13), value: 10 }),
            (vec![0,29]               , TokNewLine { location: (16, 17) }),
            (vec![0,30]               , TokMacroPrint { location: (17, 3) }),
            (vec![0,30,0]             , TokExpression),
            (vec![0,30,0,0]           , TokCompOp { location: (17, 11), op_name: "<".to_string() }),
            (vec![0,30,0,0,0]         , TokInt { location: (17, 9), value: 15 }),
            (vec![0,30,0,0,1]         , TokInt { location: (17, 12), value: 10 }),
            (vec![0,31]               , TokNewLine { location: (17, 16) }),
            (vec![0,32]               , TokMacroPrint { location: (18, 3) }),
            (vec![0,32,0]             , TokExpression),
            (vec![0,32,0,0]           , TokCompOp { location: (18, 10), op_name: "<".to_string() }),
            (vec![0,32,0,0,0]         , TokInt { location: (18, 9), value: 8 }),
            (vec![0,32,0,0,1]         , TokInt { location: (18, 11), value: 10 }),
            (vec![0,33]               , TokNewLine { location: (18, 15) }),
            (vec![0,34]               , TokMacroPrint { location: (19, 3) }),
            (vec![0,34,0]             , TokExpression),
            (vec![0,34,0,0]           , TokCompOp { location: (19, 11), op_name: "lt".to_string() }),
            (vec![0,34,0,0,0]         , TokInt { location: (19, 9), value: 10 }),
            (vec![0,34,0,0,1]         , TokInt { location: (19, 13), value: 10 }),
            (vec![0,35]               , TokNewLine { location: (19, 17) }),
            (vec![0,36]               , TokMacroPrint { location: (20, 3) }),
            (vec![0,36,0]             , TokExpression),
            (vec![0,36,0,0]           , TokCompOp { location: (20, 11), op_name: "lt".to_string() }),
            (vec![0,36,0,0,0]         , TokInt { location: (20, 9), value: 15 }),
            (vec![0,36,0,0,1]         , TokInt { location: (20, 13), value: 10 }),
            (vec![0,37]               , TokNewLine { location: (20, 17) }),
            (vec![0,38]               , TokMacroPrint { location: (21, 3) }),
            (vec![0,38,0]             , TokExpression),
            (vec![0,38,0,0]           , TokCompOp { location: (21, 10), op_name: "lt".to_string() }),
            (vec![0,38,0,0,0]         , TokInt { location: (21, 9), value: 8 }),
            (vec![0,38,0,0,1]         , TokInt { location: (21, 12), value: 10 }),
            (vec![0,39]               , TokNewLine { location: (21, 16) }),
            (vec![0,40]               , TokMacroPrint { location: (22, 3) }),
            (vec![0,40,0]             , TokExpression),
            (vec![0,40,0,0]           , TokCompOp { location: (22,11), op_name: ">".to_string() }),
            (vec![0,40,0,0,0]         , TokInt { location: (22, 9), value: 10 }),
            (vec![0,40,0,0,1]         , TokInt { location: (22, 12), value: 10 }),
            (vec![0,41]               , TokNewLine { location: (22, 16) }),
            (vec![0,42]               , TokMacroPrint { location: (23, 3) }),
            (vec![0,42,0]             , TokExpression),
            (vec![0,42,0,0]           , TokCompOp { location: (23, 11), op_name: ">".to_string() }),
            (vec![0,42,0,0,0]         , TokInt { location: (23, 9), value: 15 }),
            (vec![0,42,0,0,1]         , TokInt { location: (23, 12), value: 10 }),
            (vec![0,43]               , TokNewLine { location: (23, 16) }),
            (vec![0,44]               , TokMacroPrint { location: (24, 3) }),
            (vec![0,44,0]             , TokExpression),
            (vec![0,44,0,0]           , TokCompOp { location: (24, 10), op_name: ">".to_string() }),
            (vec![0,44,0,0,0]         , TokInt { location: (24, 9), value: 8 }),
            (vec![0,44,0,0,1]         , TokInt { location: (24, 11), value: 10 }),
            (vec![0,45]               , TokNewLine { location: (24, 15) }),
            (vec![0,46]               , TokMacroPrint { location: (25, 3) }),
            (vec![0,46,0]             , TokExpression),
            (vec![0,46,0,0]           , TokCompOp { location: (25, 11), op_name: "gt".to_string() }),
            (vec![0,46,0,0,0]         , TokInt { location: (25, 9), value: 10 }),
            (vec![0,46,0,0,1]         , TokInt { location: (25, 13), value: 10 }),
            (vec![0,47]               , TokNewLine { location: (25, 17) }),
            (vec![0,48]               , TokMacroPrint { location: (26, 3) }),
            (vec![0,48,0]             , TokExpression),
            (vec![0,48,0,0]           , TokCompOp { location: (26, 11), op_name: "gt".to_string() }),
            (vec![0,48,0,0,0]         , TokInt { location: (26, 9), value: 15 }),
            (vec![0,48,0,0,1]         , TokInt { location: (26, 13), value: 10 }),
            (vec![0,49]               , TokNewLine { location: (26, 17) }),
            (vec![0,50]               , TokMacroPrint { location: (27, 3) }),
            (vec![0,50,0]             , TokExpression),
            (vec![0,50,0,0]           , TokCompOp { location: (27, 10), op_name: "gt".to_string() }),
            (vec![0,50,0,0,0]         , TokInt { location: (27, 9), value: 8 }),
            (vec![0,50,0,0,1]         , TokInt { location: (27, 12), value: 10 }),
            (vec![0,51]               , TokNewLine { location: (27, 16) }),
            (vec![0,52]               , TokMacroPrint { location: (28, 3) }),
            (vec![0,52,0]             , TokExpression),
            (vec![0,52,0,0]           , TokCompOp { location: (28, 10), op_name: "neq".to_string() }),
            (vec![0,52,0,0,0]         , TokInt { location: (28, 9), value: 5 }),
            (vec![0,52,0,0,1]         , TokInt { location: (28, 13), value: 5 }),
            (vec![0,53]               , TokNewLine { location: (28, 16) }),
            (vec![0,54]               , TokMacroPrint { location: (29, 3) }),
            (vec![0,54,0]             , TokExpression),
            (vec![0,54,0,0]           , TokCompOp { location: (29, 14), op_name: "neq".to_string() }),
            (vec![0,54,0,0,0]         , TokInt { location: (29, 9), value: 12345 }),
            (vec![0,54,0,0,1]         , TokInt { location: (29, 17), value: 1234 }),
            (vec![0,55]               , TokNewLine { location: (29, 23) }),
            (vec![0,56]               , TokMacroPrint { location: (30, 3) }),
            (vec![0,56,0]             , TokExpression),
            (vec![0,56,0,0]           , TokCompOp { location: (30, 10), op_name: "!=".to_string() }),
            (vec![0,56,0,0,0]         , TokInt { location: (30, 9), value: 5 }),
            (vec![0,56,0,0,1]         , TokInt { location: (30, 12), value: 5 }),
            (vec![0,57]               , TokNewLine { location: (30, 15) }),
            (vec![0,58]               , TokMacroPrint { location: (31, 3) }),
            (vec![0,58,0]             , TokExpression),
            (vec![0,58,0,0]           , TokCompOp { location: (31, 15), op_name: "!=".to_string() }),
            (vec![0,58,0,0,0]         , TokInt { location: (31, 9), value: 12345 }),
            (vec![0,58,0,0,1]         , TokInt { location: (31, 18), value: 1234 }),
            (vec![0,59]               , TokNewLine { location: (31, 24) }),
            (vec![0,60]               , TokMacroPrint { location: (32, 3) }),
            (vec![0,60,0]             , TokExpression),
            (vec![0,60,0,0]           , TokCompOp { location: (32, 16), op_name: "==".to_string() }),
            (vec![0,60,0,0,0]         , TokString { location: (32, 9), value: "hallo".to_string() }),
            (vec![0,60,0,0,1]         , TokString { location: (32, 18), value: "hallo".to_string() }),
            (vec![0,61]               , TokNewLine { location: (32, 27) }),
            (vec![0,62]               , TokMacroPrint { location: (33, 3) }),
            (vec![0,62,0]             , TokExpression),
            (vec![0,62,0,0]           , TokCompOp { location: (33, 16), op_name: "==".to_string() }),
            (vec![0,62,0,0,0]         , TokString { location: (33, 9), value: "hallo".to_string() }),
            (vec![0,62,0,0,1]         , TokString { location: (33, 18), value: "hallo2".to_string() }),
            (vec![0,63]               , TokNewLine { location: (33, 28) }),
        );

        test_expected(expected, ast);
    }

    #[test]
    fn string_expressions_test() {
        let ast = test_ast("::Start\n<<print \"1234\">>\n<<print \"hallo\">>\n<<print 'hallo'>>\n<<print \"hal\"+'lo'>>\n<<print \"hal\"+\"lo\">>\n<<set $var to \"hallo\">>\n<<print $var>>\n<<print \"hallo \"+$var>>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (1, 3), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (2, 3) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokString { location: (2, 9), value: "1234".to_string() }),
            (vec![0,1]                , TokNewLine { location: (2, 17) }),
            (vec![0,2]                , TokMacroPrint { location: (3, 3) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokString { location: (3, 9), value: "hallo".to_string() }),
            (vec![0,3]                , TokNewLine { location: (3, 18) }),
            (vec![0,4]                , TokMacroPrint { location: (4, 3) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokString { location: (4, 9), value: "hallo".to_string() }),
            (vec![0,5]                , TokNewLine { location: (4, 18) }),
            (vec![0,6]                , TokMacroPrint { location: (5, 3) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokNumOp { location: (5, 14), op_name: "+".to_string() }),
            (vec![0,6,0,0,0]          , TokString { location: (5, 9), value: "hal".to_string() }),
            (vec![0,6,0,0,1]          , TokString { location: (5, 15), value: "lo".to_string() }),
            (vec![0,7]                , TokNewLine { location: (5, 21) }),
            (vec![0,8]                , TokMacroPrint { location: (6, 3) }),
            (vec![0,8,0]              , TokExpression),
            (vec![0,8,0,0]            , TokNumOp { location: (6, 14), op_name: "+".to_string() }),
            (vec![0,8,0,0,0]          , TokString { location: (6, 9), value: "hal".to_string() }),
            (vec![0,8,0,0,1]          , TokString { location: (6, 15), value: "lo".to_string() }),
            (vec![0,9]                , TokNewLine { location: (6, 21) }),
            (vec![0,10]               , TokAssign { location: (7, 7), var_name: "$var".to_string(), op_name: "to".to_string() }),
            (vec![0,10,0]             , TokExpression),
            (vec![0,10,0,0]           , TokString { location: (7, 15), value: "hallo".to_string() }),
            (vec![0,11]               , TokNewLine { location: (7, 24) }),
            (vec![0,12]               , TokMacroPrint { location: (8, 3) }),
            (vec![0,12,0]             , TokExpression),
            (vec![0,12,0,0]           , TokVariable { location: (8, 9), name: "$var".to_string() }),
            (vec![0,13]               , TokNewLine { location: (8, 15) }),
            (vec![0,14]               , TokMacroPrint { location: (9, 3) }),
            (vec![0,14,0]             , TokExpression),
            (vec![0,14,0,0]           , TokNumOp { location: (9, 17), op_name: "+".to_string() }),
            (vec![0,14,0,0,0]         , TokString { location: (9, 9), value: "hallo ".to_string() }),
            (vec![0,14,0,0,1]         , TokVariable { location: (9, 18), name: "$var".to_string() }),
            (vec![0,15]               , TokNewLine { location: (9, 24) }),
        );

        test_expected(expected, ast);
    }

    #[test]
    fn misc_expressions_test() {
        let ast = test_ast("::Start\n<<print random(1,100)+2>>\n<<print 5*3>7+3 and 5lte8>>\n<<print 15>10 or 4lte1>>\n<<if $var is 50>>fifty<<else if $var>50>>not fifty<<else>>not fifty!!<<endif>>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (1, 3), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (2, 3) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokNumOp { location: (2, 22), op_name: "+".to_string() }),
            (vec![0,0,0,0,0]          , TokFunction { location: (2, 9), name: "random".to_string() }),
            (vec![0,0,0,0,0,0]        , TokExpression),
            (vec![0,0,0,0,0,0,0]      , TokInt { location: (2, 16), value: 1 }),
            (vec![0,0,0,0,0,1]        , TokExpression),
            (vec![0,0,0,0,0,1,0]      , TokInt { location: (2, 18), value: 100 }),
            (vec![0,0,0,0,1]          , TokInt { location: (2, 23), value: 2 }),
            (vec![0,1]                , TokNewLine { location: (2, 26) }),
            (vec![0,2]                , TokMacroPrint { location: (3, 3) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokLogOp { location: (3, 17), op_name: "and".to_string() }),
            (vec![0,2,0,0,0]          , TokCompOp { location: (3, 12), op_name: ">".to_string() }),
            (vec![0,2,0,0,0,0]        , TokNumOp { location: (3, 10), op_name: "*".to_string() }),
            (vec![0,2,0,0,0,0,0]      , TokInt { location: (3, 9), value: 5 }),
            (vec![0,2,0,0,0,0,1]      , TokInt { location: (3, 11), value: 3 }),
            (vec![0,2,0,0,0,1]        , TokNumOp { location: (3, 14), op_name: "+".to_string() }),
            (vec![0,2,0,0,0,1,0]      , TokInt { location: (3, 13), value: 7 }),
            (vec![0,2,0,0,0,1,1]      , TokInt { location: (3, 15), value: 3 }),
            (vec![0,2,0,0,1]          , TokCompOp { location: (3,22), op_name: "lte".to_string() }),
            (vec![0,2,0,0,1,0]        , TokInt { location: (3, 21), value: 5 }),
            (vec![0,2,0,0,1,1]        , TokInt { location: (3, 25), value: 8 }),
            (vec![0,3]                , TokNewLine { location: (3, 28) }),
            (vec![0,4]                , TokMacroPrint { location: (4, 3) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokLogOp { location: (4, 15), op_name: "or".to_string() }),
            (vec![0,4,0,0,0]          , TokCompOp { location: (4, 11), op_name: ">".to_string() }),
            (vec![0,4,0,0,0,0]        , TokInt { location: (4, 9), value: 15 }),
            (vec![0,4,0,0,0,1]        , TokInt { location: (4, 12), value: 10 }),
            (vec![0,4,0,0,1]          , TokCompOp { location: (4, 19), op_name: "lte".to_string() }),
            (vec![0,4,0,0,1,0]        , TokInt { location: (4, 18), value: 4 }),
            (vec![0,4,0,0,1,1]        , TokInt { location: (4, 22), value: 1 }),
            (vec![0,5]                , TokNewLine { location: (4, 25) }),
            (vec![0,6]                , TokMacroIf { location: (5, 3) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokCompOp { location: (5, 11), op_name: "is".to_string() }),
            (vec![0,6,0,0,0]          , TokVariable { location: (5, 6), name: "$var".to_string() }),
            (vec![0,6,0,0,1]          , TokInt { location: (5, 14), value: 50 }),
            (vec![0,6,1]              , TokText { location: (5, 18), text: "fifty".to_string() }),
            (vec![0,7]                , TokMacroElseIf { location: (5, 25) }),
            (vec![0,7,0]              , TokExpression),
            (vec![0,7,0,0]            , TokCompOp { location: (5, 37), op_name: ">".to_string() }),
            (vec![0,7,0,0,0]          , TokVariable { location: (5, 33), name: "$var".to_string() }),
            (vec![0,7,0,0,1]          , TokInt { location: (5, 38), value: 50 }),
            (vec![0,7,1]              , TokText { location: (5, 42), text: "not fifty".to_string() }),
            (vec![0,8]                , TokMacroElse { location: (5, 53) }),
            (vec![0,8,0]              , TokText { location: (5, 59), text: "not fifty!!".to_string() }),
            (vec![0,9]                , TokMacroEndIf { location: (5, 72) }),
            (vec![0,10]               , TokNewLine { location: (5, 79) }),
        );

        test_expected(expected, ast);
    }
}
