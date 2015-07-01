//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

use std::fmt::{Debug, Display, Formatter, Result, Write};

use config::Config;
use backend::zcode::zfile;
use backend::zcode::zfile::{ZOP, Type};
use frontend::codegen;
use frontend::expressionparser;
use frontend::lexer::Token;
use frontend::lexer::Token::{TokMacroIf, TokMacroElseIf, TokExpression, TokPassage};

//==============================
// ast

pub struct ASTBuilder<'a> {
    path: Vec<usize>,
    is_in_if_expression: bool,
    cfg: &'a Config,
    ast: AST,
}

/// The AST holds only a vectory of all passage-nodes
pub struct AST {
    passages: Vec<ASTNode>,
}

/// the parser creates an iterator with these enums to build the AST
pub enum ASTOperation {
    AddPassage(Token),
    AddChild(Token),
    ChildDown(Token),
    Up,
    UpChild(Token),
    UpChildDown(Token),
    UpSpecial,
}

impl<'a> ASTBuilder<'a> {
    pub fn new(cfg: &'a Config) -> ASTBuilder {
        ASTBuilder {
            path: Vec::new(),
            is_in_if_expression: false,
            cfg: cfg,
            ast: AST {
                passages: Vec::new()
            }
        }
    }

    /// gets an iterator of ASTOperation and returns an AST
    pub fn build<I: Iterator<Item=ASTOperation>>(mut self, ops: I) -> AST {
        for op in ops {
            self.operation(op);
        }

        self.parse_expressions();
        self.ast
    }

    /// ASTOperation-enum -> function match
    pub fn operation(&mut self, op: ASTOperation) {
        use self::ASTOperation::*;
        match op {
            AddPassage(passage) => self.add_passage(passage),
            AddChild(child) => self.add_child(child),
            ChildDown(child) => self.child_down(child),
            Up => self.up(),
            UpChild(child) => self.up_child(child),
            UpChildDown(child) => self.up_child_down(child),
            UpSpecial => self.up_special(),
        }
    }

    /// goes through the whole tree and parse the expressions
    fn parse_expressions(&mut self) {
        for child in &mut self.ast.passages {
            child.parse_expressions(&self.cfg);
        }
    }

    /// adds a passage to the path in the ast
    pub fn add_passage(&mut self, token: Token) {
        self.path.clear();
        let ast_count_passages = self.ast.count_childs(self.path.to_vec());

        let node = ASTNode::Passage(NodePassage { category: token, childs: Vec::new() });
        self.ast.passages.push(node);

        self.path.push(ast_count_passages);
    }

    /// adds a child to the path in the ast
    pub fn add_child(&mut self, token: Token) {
        if let Some(index) = self.path.first() {
            let mut new_path: Vec<usize> = self.path.to_vec();
            new_path.remove(0);
            self.ast.passages[*index].add_child(new_path, token);
        } else {
            self.ast.passages.push(ASTNode::Default(NodeDefault { category: token, childs: Vec::new() }));
        }
    }

    /// adds a child an goees one child down
    pub fn child_down(&mut self, token: Token) {
        //
        if token.clone().is_same_token(&TokMacroIf { location: (0, 0) }) ||
           token.clone().is_same_token(&TokMacroElseIf { location: (0, 0) }) {
            self.is_in_if_expression = true;
        }

        let ast_count_childs = self.ast.count_childs(self.path.to_vec());
        self.add_child(token);
        self.path.push(ast_count_childs);
    }

    /// adds one child and goes down. adds snd child and goes down.
    pub fn two_childs_down(&mut self, child1: Token, child2: Token) {
        self.child_down(child1);
        self.child_down(child2);
    }

    /// goes one lvl up
    pub fn up(&mut self) {
        self.path.pop();
    }

    /// special up of the if-expression
    pub fn up_special(&mut self) {
        if !self.is_in_if_expression {
            self.path.pop();
        } else {
            self.is_in_if_expression = false;
        }
    }

    /// goes one lvl up and adds and child
    pub fn up_child(&mut self, token: Token) {
        self.up();
        self.add_child(token);
    }

    /// goes one lvl up, adds an child and goes one lvl down
    pub fn up_child_down(&mut self, token: Token) {
        self.up();
        self.child_down(token);
    }

    /// goes two lvl up
    pub fn two_up(&mut self) {
        self.up();
        self.up();
    }
}

impl AST {
    /// convert ast to zcode
    pub fn to_zcode(&self, out: &mut zfile::Zfile) {
        let mut manager = codegen::CodeGenManager::new();

        // adds a vec of passagenames to the manager
        manager.passages = self.passage_nodes_to_string();

        // Insert temp variables for internal calculations
        manager.symbol_table.insert_new_symbol("int0", Type::Integer);

        let mut code: Vec<ZOP> = vec![];
        for child in &self.passages {
            for instr in codegen::gen_zcode(child, out, &mut manager) {
                code.push(instr);
            }
        }
        out.emit(code);
    }

    /// counts the childs of the path in the asts
    pub fn count_childs(&self, path: Vec<usize>) -> usize {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            self.passages[*index].count_childs(new_path)
        } else {
            self.passages.len()
        }
    }

    /// checks in the ast if there is the token "token"
    pub fn is_specific_token(&self, token: Token, path: Vec<usize>) -> bool {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            self.passages[*index].is_specific_token(token, new_path)
        } else {
            false
        }
    }

    /// cycle through all passages an returns a vector with all passage-titles
    fn passage_nodes_to_string(&self) -> Vec<String> {
        let mut passages: Vec<String> = Vec::new();
        for child in &self.passages {
            match child.category() {
                TokPassage {ref name, .. } => {
                    passages.push(name.clone());
                }
                _ => ()
            }
        }

        passages
    }
}

/// To Print the AST
impl Debug for AST {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(f.write_str("Abstract Syntax Tree: \n"));
        for child in &self.passages {
            try!(child.fmt(f));
        }
        Ok(())
    }
}

// ================================
// node types
#[derive(Clone)]
pub enum ASTNode {
    Default (NodeDefault),
    Passage (NodePassage)
}

#[derive(Clone)]
pub struct NodePassage {
    pub category: Token,
    pub childs: Vec<ASTNode>,
    /*tags: Vec<ASTNode>*/
}

#[derive(Clone)]
pub struct NodeDefault {
    pub category: Token,
    pub childs: Vec<ASTNode>
}

impl ASTNode {
    /// adds an child to the path in the ast
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

    /// counts the childs of the current path in the ast
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

    /// checks the current path if there is the token "token"
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
                    token.is_same_token(&node.category)
                },
                &ASTNode::Passage(ref node) => {
                    token.is_same_token(&node.category)
                },
            }
        }
    }

    /// returns the category-Token of the current node
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

    /// returns e vector of all childs of the current node
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

    /// wraps the ASTNode to NodeDefault if it is an NodeDefault
    pub fn as_default(&self) -> &NodeDefault {
        match self {
            &ASTNode::Default(ref def) => def,
            _ => panic!("Node cannot be unwrapped as NodeDefault!")
        }
    }

    /// goes through the whole tree and parse the expressions
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

    /// creates an ast from the inputs str
    fn test_ast(input: &str) -> AST {
        let cfg = Config::default_config();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(input.to_string().into_bytes());
        let tokens = lexer::lex(&cfg, &mut cursor);
        let parser = parser::Parser::new(&cfg);
        let ast_ops = parser.parse(tokens.inspect(|ref token| {
            println!("{:?}", token);
        }));

        let ast_builder = ASTBuilder::new(&cfg);
        ast_builder.build(ast_ops)
    }

    /// checks expected
    fn test_expected(expected: Vec<(Vec<usize>, Token)>, ast: AST) {
        for item in expected.iter() {
            let b = ast.is_specific_token(item.1.clone(), item.0.to_vec());
            if b == false {
                println!("{:?}", ast);
            }
            assert!(ast.is_specific_token(item.1.clone(), item.0.to_vec()));
        }
    }

    #[test]
    fn text_test() {
        let ast = test_ast("::Start\nTestText\nTestNextLine\n::NextPassage\nOtherText");

        let expected = vec!(
            (vec![0]  , TokPassage {location: (0, 0), name: "Start".to_string()}),
            (vec![0,0], TokText {location: (0, 0), text: "TestText".to_string()}),
            (vec![0,1], TokNewLine {location: (0, 0)} ),
            (vec![0,2], TokText {location: (0, 0), text: "nTestNextLine".to_string()}),
            (vec![0,1], TokNewLine {location: (0, 0)}),
            (vec![1]  , TokPassage {location: (0, 0), name: "NextPassage".to_string()}),
            (vec![1,0], TokText {location: (0, 0), text: "OtherText".to_string()}),

        );

        test_expected(expected, ast);
    }

    #[test]
    fn num_expressions_test() {
        let ast = test_ast("::Start\n<<print -12345>>\n<<print 5>>\n<<print 32767>>\n<<print 1*2*3*4*5*6*7>>\n<<print 1*2+3*4+5*6+7>>\n<<print 1*2-3*4-5*6-7>>\n<<print 256/8/4/8>>\n<<print 6300/5/7/9/10>>\n<<print 6300/5/7/-9/10>>\n<<print 1-3>>\n<<print -2+2>>\n<<print (1+2)*(3--4)>>\n<<print (1+2)*(3+4)*(5+6)*(7+8)>>\n<<print (1-2)*(3-4)*(5-6)*(7-8)>>\n<<print ((1-2)*(3+4))*(5-6)*(7-8)>>\n<<print (2*9)/(-7)>>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (0, 0), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokUnaryMinus { location: (0, 0) }),
            (vec![0,0,0,0,0]          , TokInt { location: (0, 0), value: 12345 }),
            (vec![0,1]                , TokNewLine { location: (0, 0) }),
            (vec![0,2]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokInt { location: (0, 0), value: 5 }),
            (vec![0,3]                , TokNewLine { location: (0, 0) }),
            (vec![0,4]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokInt { location: (0, 0), value: 32767 }),
            (vec![0,5]                , TokNewLine { location: (0, 0) }),
            (vec![0,6]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,6,0,0,0]          , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0]        , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0,0]      , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0,0,0]    , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0,0,0,0]  , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,6,0,0,0,0,0,0,0,0], TokInt { location: (0, 0), value: 1 }),
            (vec![0,6,0,0,0,0,0,0,0,1], TokInt { location: (0, 0), value: 2 }),
            (vec![0,6,0,0,0,0,0,0,1]  , TokInt { location: (0, 0), value: 3 }),
            (vec![0,6,0,0,0,0,0,1]    , TokInt { location: (0, 0), value: 4 }),
            (vec![0,6,0,0,0,0,1]      , TokInt { location: (0, 0), value: 5 }),
            (vec![0,6,0,0,0,1]        , TokInt { location: (0, 0), value: 6 }),
            (vec![0,6,0,0,1]          , TokInt { location: (0, 0), value: 7 }),
            (vec![0,7]                , TokNewLine { location: (0, 0) }),
            (vec![0,8]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,8,0]              , TokExpression),
            (vec![0,8,0,0]            , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,8,0,0,0]          , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,8,0,0,0,0]        , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,8,0,0,0,0,0]      , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,8,0,0,0,0,0,0]    , TokInt { location: (0, 0), value: 1 }),
            (vec![0,8,0,0,0,0,0,1]    , TokInt { location: (0, 0), value: 2 }),
            (vec![0,8,0,0,0,0,1]      , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,8,0,0,0,0,1,0]    , TokInt { location: (0, 0), value: 3 }),
            (vec![0,8,0,0,0,0,1,1]    , TokInt { location: (0, 0), value: 4 }),
            (vec![0,8,0,0,0,1]        , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,8,0,0,0,1,0]      , TokInt { location: (0, 0), value: 5 }),
            (vec![0,8,0,0,0,1,1]      , TokInt { location: (0, 0), value: 6 }),
            (vec![0,8,0,0,1]          , TokInt { location: (0, 0), value: 7 }),
            (vec![0,9]                , TokNewLine { location: (0, 0) }),
            (vec![0,10]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,10,0]             , TokExpression),
            (vec![0,10,0,0]           , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,10,0,0,0]         , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,10,0,0,0,0]       , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,10,0,0,0,0,0]     , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,10,0,0,0,0,0,0]   , TokInt { location: (0, 0), value: 1 }),
            (vec![0,10,0,0,0,0,0,1]   , TokInt { location: (0, 0), value: 2 }),
            (vec![0,10,0,0,0,0,1]     , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,10,0,0,0,0,1,0]   , TokInt { location: (0, 0), value: 3 }),
            (vec![0,10,0,0,0,0,1,1]   , TokInt { location: (0, 0), value: 4 }),
            (vec![0,10,0,0,0,1]       , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,10,0,0,0,1,0]     , TokInt { location: (0, 0), value: 5 }),
            (vec![0,10,0,0,0,1,1]     , TokInt { location: (0, 0), value: 6 }),
            (vec![0,10,0,0,1]         , TokInt { location: (0, 0), value: 7 }),
            (vec![0,11]               , TokNewLine { location: (0, 0) }),
            (vec![0,12]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,12,0]             , TokExpression),
            (vec![0,12,0,0]           , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,12,0,0,0]         , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,12,0,0,0,0]       , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,12,0,0,0,0,0]     , TokInt { location: (0, 0), value: 256 }),
            (vec![0,12,0,0,0,0,1]     , TokInt { location: (0, 0), value: 8 }),
            (vec![0,12,0,0,0,1]       , TokInt { location: (0, 0), value: 4 }),
            (vec![0,12,0,0,1]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,13]               , TokNewLine { location: (0, 0) }),
            (vec![0,14]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,14,0]             , TokExpression),
            (vec![0,14,0,0]           , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,14,0,0,0]         , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,14,0,0,0,0]       , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,14,0,0,0,0,0]     , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,14,0,0,0,0,0,0]   , TokInt { location: (0, 0), value: 6300 }),
            (vec![0,14,0,0,0,0,0,1]   , TokInt { location: (0, 0), value: 5 }),
            (vec![0,14,0,0,0,0,1]     , TokInt { location: (0, 0), value: 7 }),
            (vec![0,14,0,0,0,1]       , TokInt { location: (0, 0), value: 9 }),
            (vec![0,14,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,15]               , TokNewLine { location: (0, 0) }),
            (vec![0,16]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,16,0]             , TokExpression),
            (vec![0,16,0,0]           , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,16,0,0,0]         , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,16,0,0,0,0]       , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,16,0,0,0,0,0]     , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,16,0,0,0,0,0,0]   , TokInt { location: (0, 0), value: 6300 }),
            (vec![0,16,0,0,0,0,0,1]   , TokInt { location: (0, 0), value: 5 }),
            (vec![0,16,0,0,0,0,1]     , TokInt { location: (0, 0), value: 7 }),
            (vec![0,16,0,0,0,1]       , TokUnaryMinus { location: (0, 0) }),
            (vec![0,16,0,0,0,0,1]     , TokInt { location: (0, 0), value: 9 }),
            (vec![0,16,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,17]               , TokNewLine { location: (0, 0) }),
            (vec![0,18]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,18,0]             , TokExpression),
            (vec![0,18,0,0]           , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,18,0,0,0]         , TokInt { location: (0, 0), value: 1 }),
            (vec![0,18,0,0,1]         , TokInt { location: (0, 0), value: 3 }),
            (vec![0,19]               , TokNewLine { location: (0, 0) }),
            (vec![0,20]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,20,0]             , TokExpression),
            (vec![0,20,0,0]           , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,20,0,0,0]         , TokUnaryMinus { location: (0, 0) }),
            (vec![0,20,0,0,0,0]       , TokInt { location: (0, 0), value: 2 }),
            (vec![0,20,0,0,1]         , TokInt { location: (0, 0), value: 2 }),
            (vec![0,21]               , TokNewLine { location: (0, 0) }),
            (vec![0,22]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,22,0]             , TokExpression),
            (vec![0,22,0,0]           , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,22,0,0,0]         , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,22,0,0,0,0]       , TokInt { location: (0, 0), value: 1 }),
            (vec![0,22,0,0,0,1]       , TokInt { location: (0, 0), value: 2 }),
            (vec![0,22,0,0,1]         , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,22,0,0,1,0]       , TokInt { location: (0, 0), value: 3 }),
            (vec![0,22,0,0,1,1]       , TokUnaryMinus { location: (0, 0) }),
            (vec![0,22,0,0,1,1,0]     , TokInt { location: (0, 0), value: 4 }),
            (vec![0,23]               , TokNewLine { location: (0, 0) }),
            (vec![0,24]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,24,0]             , TokExpression),
            (vec![0,24,0,0]           , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,24,0,0,0]         , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,24,0,0,0,0]       , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,24,0,0,0,0,0]     , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,24,0,0,0,0,0,0]   , TokInt { location: (0, 0), value: 1 }),
            (vec![0,24,0,0,0,0,0,1]   , TokInt { location: (0, 0), value: 2 }),
            (vec![0,24,0,0,0,0,1]     , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,24,0,0,0,0,1,0]   , TokInt { location: (0, 0), value: 3 }),
            (vec![0,24,0,0,0,0,1,1]   , TokInt { location: (0, 0), value: 4 }),
            (vec![0,24,0,0,0,1]       , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,24,0,0,0,1,0]     , TokInt { location: (0, 0), value: 5 }),
            (vec![0,24,0,0,0,1,1]     , TokInt { location: (0, 0), value: 6 }),
            (vec![0,24,0,0,1]         , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,24,0,0,1,0]       , TokInt { location: (0, 0), value: 7 }),
            (vec![0,24,0,0,1,1]       , TokInt { location: (0, 0), value: 8 }),
            (vec![0,25]               , TokNewLine { location: (0, 0) }),
            (vec![0,26]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,26,0]             , TokExpression),
            (vec![0,26,0,0]           , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,26,0,0,0]         , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,26,0,0,0,0]       , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,26,0,0,0,0,0]     , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,26,0,0,0,0,0,0]   , TokInt { location: (0, 0), value: 1 }),
            (vec![0,26,0,0,0,0,0,1]   , TokInt { location: (0, 0), value: 2 }),
            (vec![0,26,0,0,0,0,1]     , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,26,0,0,0,0,1,0]   , TokInt { location: (0, 0), value: 3 }),
            (vec![0,26,0,0,0,0,1,1]   , TokInt { location: (0, 0), value: 4 }),
            (vec![0,26,0,0,0,1]       , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,26,0,0,0,1,0]     , TokInt { location: (0, 0), value: 5 }),
            (vec![0,26,0,0,0,1,1]     , TokInt { location: (0, 0), value: 6 }),
            (vec![0,26,0,0,1]         , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,26,0,0,1,0]       , TokInt { location: (0, 0), value: 7 }),
            (vec![0,26,0,0,1,1]       , TokInt { location: (0, 0), value: 8 }),
            (vec![0,27]               , TokNewLine { location: (0, 0) }),
            (vec![0,28]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,28,0]             , TokExpression),
            (vec![0,28,0,0]           , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,28,0,0,0]         , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,28,0,0,0,0]       , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,28,0,0,0,0,0]     , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,28,0,0,0,0,0,0]   , TokInt { location: (0, 0), value: 1 }),
            (vec![0,28,0,0,0,0,0,1]   , TokInt { location: (0, 0), value: 2 }),
            (vec![0,28,0,0,0,0,1]     , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,28,0,0,0,0,1,0]   , TokInt { location: (0, 0), value: 3 }),
            (vec![0,28,0,0,0,0,1,1]   , TokInt { location: (0, 0), value: 4 }),
            (vec![0,28,0,0,0,1]       , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,28,0,0,0,1,0]     , TokInt { location: (0, 0), value: 5 }),
            (vec![0,28,0,0,0,1,1]     , TokInt { location: (0, 0), value: 6 }),
            (vec![0,28,0,0,1]         , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,28,0,0,1,0]       , TokInt { location: (0, 0), value: 7 }),
            (vec![0,28,0,0,1,1]       , TokInt { location: (0, 0), value: 8 }),
            (vec![0,29]               , TokNewLine { location: (0, 0) }),
            (vec![0,30]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,30,0]             , TokExpression),
            (vec![0,30,0,0]           , TokNumOp { location: (0, 0), op_name: "/".to_string() }),
            (vec![0,30,0,0,0]         , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,30,0,0,0,0]       , TokInt { location: (0, 0), value: 2 }),
            (vec![0,30,0,0,0,1]       , TokInt { location: (0, 0), value: 9 }),
            (vec![0,30,0,0,1]         , TokUnaryMinus { location: (0, 0) }),
            (vec![0,30,0,0,1,0]       , TokInt { location: (0, 0), value: 7 }),
            (vec![0,31]               , TokNewLine { location: (0, 0) }),
        );

        test_expected(expected, ast);
    }

    #[test]
    fn log_expressions_test() {
        let ast = test_ast("::Start\n<<print false>>\n<<print true>>\n<<print not false>>\n<<print not true>>\n<<print not-5>>\n<<print not5>>\n<<print not0>>\n<<print true and true>>\n<<print true and false>>\n<<print false and true>>\n<<print false and false>>\n<<print true or true>>\n<<print true or false>>\n<<print false or true>>\n<<print false or false>>\n<<print false or true and true>>\n<<print false or true or false>>\n<<print true or false and true and false or true>>\n<<print (true or false) and false>>\n<<print (true or false) and (true or true)>>\n<<print (true and true)>>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (0, 0), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,1]                , TokNewLine { location: (0, 0) }),
            (vec![0,2]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,3]                , TokNewLine { location: (0, 0) }),
            (vec![0,4]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokLogOp { location: (0, 0), op_name: "not".to_string() }),
            (vec![0,4,0,0,0]          , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,5]                , TokNewLine { location: (0, 0) }),
            (vec![0,6]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokLogOp { location: (0, 0), op_name: "not".to_string() }),
            (vec![0,6,0,0,0]          , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,7]                , TokNewLine { location: (0, 0) }),
            (vec![0,8]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,8,0]              , TokExpression),
            (vec![0,8,0,0]            , TokLogOp { location: (0, 0), op_name: "not".to_string() }),
            (vec![0,8,0,0,0]          , TokUnaryMinus { location: (0, 0) }),
            (vec![0,8,0,0,0,0]        , TokInt { location: (0, 0), value: 5 }),
            (vec![0,9]                , TokNewLine { location: (0, 0) }),
            (vec![0,10]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,10,0]             , TokExpression),
            (vec![0,10,0,0]           , TokLogOp { location: (0, 0), op_name: "not".to_string() }),
            (vec![0,10,0,0,0]         , TokInt { location: (0, 0), value: 5 }),
            (vec![0,11]               , TokNewLine { location: (0, 0) }),
            (vec![0,12]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,12,0]             , TokExpression),
            (vec![0,12,0,0]           , TokLogOp { location: (0, 0), op_name: "not".to_string() }),
            (vec![0,12,0,0,0]         , TokInt { location: (0, 0), value: 0 }),
            (vec![0,13]               , TokNewLine { location: (0, 0) }),
            (vec![0,14]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,14,0]             , TokExpression),
            (vec![0,14,0,0]           , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,14,0,0,0]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,14,0,0,1]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,15]               , TokNewLine { location: (0, 0) }),
            (vec![0,16]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,16,0]             , TokExpression),
            (vec![0,16,0,0]           , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,16,0,0,0]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,16,0,0,1]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,17]               , TokNewLine { location: (0, 0) }),
            (vec![0,18]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,18,0]             , TokExpression),
            (vec![0,18,0,0]           , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,18,0,0,0]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,18,0,0,1]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,19]               , TokNewLine { location: (0, 0) }),
            (vec![0,20]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,20,0]             , TokExpression),
            (vec![0,20,0,0]           , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,20,0,0,0]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,20,0,0,1]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,21]               , TokNewLine { location: (0, 0) }),
            (vec![0,22]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,22,0]             , TokExpression),
            (vec![0,22,0,0]           , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,22,0,0,0]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,22,0,0,1]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,23]               , TokNewLine { location: (0, 0) }),
            (vec![0,24]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,24,0]             , TokExpression),
            (vec![0,24,0,0]           , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,24,0,0,0]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,24,0,0,1]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,25]               , TokNewLine { location: (0, 0) }),
            (vec![0,26]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,26,0]             , TokExpression),
            (vec![0,26,0,0]           , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,26,0,0,0]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,26,0,0,1]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,27]               , TokNewLine { location: (0, 0) }),
            (vec![0,28]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,28,0]             , TokExpression),
            (vec![0,28,0,0]           , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,28,0,0,0]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,28,0,0,1]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,29]               , TokNewLine { location: (0, 0) }),
            (vec![0,30]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,30,0]             , TokExpression),
            (vec![0,30,0,0]           , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,30,0,0,0]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,30,0,0,1]         , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,30,0,0,1,0]       , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,30,0,0,1,1]       , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,31]               , TokNewLine { location: (0, 0) }),
            (vec![0,32]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,32,0]             , TokExpression),
            (vec![0,32,0,0]           , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,32,0,0,0]         , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,32,0,0,0,0]       , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,32,0,0,0,1]       , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,32,0,0,1]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,33]               , TokNewLine { location: (0, 0) }),
            (vec![0,34]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,34,0]             , TokExpression),
            (vec![0,34,0,0]           , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,34,0,0,0]         , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,34,0,0,0,0]       , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,34,0,0,0,1]       , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,34,0,0,0,1,0]     , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,34,0,0,0,1,0,0]   , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,34,0,0,0,1,0,1]   , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,34,0,0,0,1,1]     , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,34,0,0,1]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,35]               , TokNewLine { location: (0, 0) }),
            (vec![0,36]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,36,0]             , TokExpression),
            (vec![0,36,0,0]           , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,36,0,0,0]         , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,36,0,0,0,0]       , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,36,0,0,0,1]       , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,36,0,0,1]         , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,37]               , TokNewLine { location: (0, 0) }),
            (vec![0,38]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,38,0]             , TokExpression),
            (vec![0,38,0,0]           , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,38,0,0,0]         , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,38,0,0,0,0]       , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,38,0,0,0,1]       , TokBoolean { location: (0, 0), value: "false".to_string() }),
            (vec![0,38,0,0,1]         , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,38,0,0,1,0]       , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,38,0,0,1,1]       , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,39]               , TokNewLine { location: (0, 0) }),
            (vec![0,40]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,40,0]             , TokExpression),
            (vec![0,40,0,0]           , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,40,0,0,0]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,40,0,0,1]         , TokBoolean { location: (0, 0), value: "true".to_string() }),
            (vec![0,41]               , TokNewLine { location: (0, 0) }),
        );

        test_expected(expected, ast);
    }

    #[test]
    fn comp_expressions_test() {
        let ast = test_ast("::Start\n<<print 5==5>>\n<<print 12345==1234>>\n<<print 5is5>>\n<<print 12345is1234>>\n<<print 10>=10>>\n<<print 15>=10>>\n<<print 8>=10>>\n<<print 10gte10>>\n<<print 15gte10>>\n<<print 8gte10>>\n<<print 10<=10>>\n<<print 15<=10>>\n<<print 8<=10>>\n<<print 10lte10>>\n<<print 8lte10>>\n<<print 15<10>>\n<<print 8<10>>\n<<print 10lt10>>\n<<print 15lt10>>\n<<print 8lt10>>\n<<print 10>10>>\n<<print 15>10>>\n<<print 8>10>>\n<<print 10gt10>>\n<<print 15gt10>>\n<<print 8gt10>>\n<<print 5neq5>>\n<<print 12345neq1234>>\n<<print \"hallo\"==\"hallo\">>\n<<print \"hallo\"==\"hallo2\">>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (0, 0), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokCompOp { location: (0, 0), op_name: "==".to_string() }),
            (vec![0,0,0,0,0]          , TokInt { location: (0, 0), value: 5 }),
            (vec![0,0,0,0,1]          , TokInt { location: (0, 0), value: 5 }),
            (vec![0,1]                , TokNewLine { location: (0, 0) }),
            (vec![0,2]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokCompOp { location: (0, 0), op_name: "==".to_string() }),
            (vec![0,2,0,0,0]          , TokInt { location: (0, 0), value: 12345 }),
            (vec![0,2,0,0,1]          , TokInt { location: (0, 0), value: 1234 }),
            (vec![0,3]                , TokNewLine { location: (0, 0) }),
            (vec![0,4]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokCompOp { location: (0, 0), op_name: "is".to_string() }),
            (vec![0,4,0,0,0]          , TokInt { location: (0, 0), value: 5 }),
            (vec![0,4,0,0,1]          , TokInt { location: (0, 0), value: 5 }),
            (vec![0,5]                , TokNewLine { location: (0, 0) }),
            (vec![0,6]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokCompOp { location: (0, 0), op_name: "is".to_string() }),
            (vec![0,6,0,0,0]          , TokInt { location: (0, 0), value: 12345 }),
            (vec![0,6,0,0,1]          , TokInt { location: (0, 0), value: 1234 }),
            (vec![0,7]                , TokNewLine { location: (0, 0) }),
            (vec![0,8]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,8,0]              , TokExpression),
            (vec![0,8,0,0]            , TokCompOp { location: (0, 0), op_name: ">=".to_string() }),
            (vec![0,8,0,0,0]          , TokInt { location: (0, 0), value: 10 }),
            (vec![0,8,0,0,1]          , TokInt { location: (0, 0), value: 10 }),
            (vec![0,9]                , TokNewLine { location: (0, 0) }),
            (vec![0,10]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,10,0]             , TokExpression),
            (vec![0,10,0,0]           , TokCompOp { location: (0, 0), op_name: ">=".to_string() }),
            (vec![0,10,0,0,0]         , TokInt { location: (0, 0), value: 15 }),
            (vec![0,10,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,11]               , TokNewLine { location: (0, 0) }),
            (vec![0,12]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,12,0]             , TokExpression),
            (vec![0,12,0,0]           , TokCompOp { location: (0, 0), op_name: ">=".to_string() }),
            (vec![0,12,0,0,0]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,12,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,13]               , TokNewLine { location: (0, 0) }),
            (vec![0,14]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,14,0]             , TokExpression),
            (vec![0,14,0,0]           , TokCompOp { location: (0, 0), op_name: "gte".to_string() }),
            (vec![0,14,0,0,0]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,14,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,15]               , TokNewLine { location: (0, 0) }),
            (vec![0,16]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,16,0]             , TokExpression),
            (vec![0,16,0,0]           , TokCompOp { location: (0, 0), op_name: "gte".to_string() }),
            (vec![0,16,0,0,0]         , TokInt { location: (0, 0), value: 15 }),
            (vec![0,16,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,17]               , TokNewLine { location: (0, 0) }),
            (vec![0,18]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,18,0]             , TokExpression),
            (vec![0,18,0,0]           , TokCompOp { location: (0, 0), op_name: "gte".to_string() }),
            (vec![0,18,0,0,0]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,18,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,19]               , TokNewLine { location: (0, 0) }),
            (vec![0,20]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,20,0]             , TokExpression),
            (vec![0,20,0,0]           , TokCompOp { location: (0, 0), op_name: "<=".to_string() }),
            (vec![0,20,0,0,0]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,20,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,21]               , TokNewLine { location: (0, 0) }),
            (vec![0,22]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,22,0]             , TokExpression),
            (vec![0,22,0,0]           , TokCompOp { location: (0, 0), op_name: "<=".to_string() }),
            (vec![0,22,0,0,0]         , TokInt { location: (0, 0), value: 15 }),
            (vec![0,22,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,23]               , TokNewLine { location: (0, 0) }),
            (vec![0,24]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,24,0]             , TokExpression),
            (vec![0,24,0,0]           , TokCompOp { location: (0, 0), op_name: "<=".to_string() }),
            (vec![0,24,0,0,0]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,24,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,25]               , TokNewLine { location: (0, 0) }),
            (vec![0,26]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,26,0]             , TokExpression),
            (vec![0,26,0,0]           , TokCompOp { location: (0, 0), op_name: "lte".to_string() }),
            (vec![0,26,0,0,0]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,26,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,27]               , TokNewLine { location: (0, 0) }),
            (vec![0,28]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,28,0]             , TokExpression),
            (vec![0,28,0,0]           , TokCompOp { location: (0, 0), op_name: "lte".to_string() }),
            (vec![0,28,0,0,0]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,28,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,29]               , TokNewLine { location: (0, 0) }),
            (vec![0,30]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,30,0]             , TokExpression),
            (vec![0,30,0,0]           , TokCompOp { location: (0, 0), op_name: "<".to_string() }),
            (vec![0,30,0,0,0]         , TokInt { location: (0, 0), value: 15 }),
            (vec![0,30,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,31]               , TokNewLine { location: (0, 0) }),
            (vec![0,32]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,32,0]             , TokExpression),
            (vec![0,32,0,0]           , TokCompOp { location: (0, 0), op_name: "<".to_string() }),
            (vec![0,32,0,0,0]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,32,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,33]               , TokNewLine { location: (0, 0) }),
            (vec![0,34]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,34,0]             , TokExpression),
            (vec![0,34,0,0]           , TokCompOp { location: (0, 0), op_name: "lte".to_string() }),
            (vec![0,34,0,0,0]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,34,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,35]               , TokNewLine { location: (0, 0) }),
            (vec![0,36]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,36,0]             , TokExpression),
            (vec![0,36,0,0]           , TokCompOp { location: (0, 0), op_name: "lte".to_string() }),
            (vec![0,36,0,0,0]         , TokInt { location: (0, 0), value: 15 }),
            (vec![0,36,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,37]               , TokNewLine { location: (0, 0) }),
            (vec![0,38]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,38,0]             , TokExpression),
            (vec![0,38,0,0]           , TokCompOp { location: (0, 0), op_name: "lte".to_string() }),
            (vec![0,38,0,0,0]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,38,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,39]               , TokNewLine { location: (0, 0) }),
            (vec![0,40]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,40,0]             , TokExpression),
            (vec![0,40,0,0]           , TokCompOp { location: (0, 0), op_name: ">".to_string() }),
            (vec![0,40,0,0,0]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,40,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,41]               , TokNewLine { location: (0, 0) }),
            (vec![0,42]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,42,0]             , TokExpression),
            (vec![0,42,0,0]           , TokCompOp { location: (0, 0), op_name: ">".to_string() }),
            (vec![0,42,0,0,0]         , TokInt { location: (0, 0), value: 15 }),
            (vec![0,42,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,43]               , TokNewLine { location: (0, 0) }),
            (vec![0,44]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,44,0]             , TokExpression),
            (vec![0,44,0,0]           , TokCompOp { location: (0, 0), op_name: ">".to_string() }),
            (vec![0,44,0,0,0]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,44,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,45]               , TokNewLine { location: (0, 0) }),
            (vec![0,46]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,46,0]             , TokExpression),
            (vec![0,46,0,0]           , TokCompOp { location: (0, 0), op_name: "gt".to_string() }),
            (vec![0,46,0,0,0]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,46,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,47]               , TokNewLine { location: (0, 0) }),
            (vec![0,48]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,48,0]             , TokExpression),
            (vec![0,48,0,0]           , TokCompOp { location: (0, 0), op_name: "gt".to_string() }),
            (vec![0,48,0,0,0]         , TokInt { location: (0, 0), value: 15 }),
            (vec![0,48,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,49]               , TokNewLine { location: (0, 0) }),
            (vec![0,50]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,50,0]             , TokExpression),
            (vec![0,50,0,0]           , TokCompOp { location: (0, 0), op_name: "gt".to_string() }),
            (vec![0,50,0,0,0]         , TokInt { location: (0, 0), value: 8 }),
            (vec![0,50,0,0,1]         , TokInt { location: (0, 0), value: 10 }),
            (vec![0,51]               , TokNewLine { location: (0, 0) }),
            (vec![0,52]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,52,0]             , TokExpression),
            (vec![0,52,0,0]           , TokCompOp { location: (0, 0), op_name: "neq".to_string() }),
            (vec![0,52,0,0,0]         , TokInt { location: (0, 0), value: 5 }),
            (vec![0,52,0,0,1]         , TokInt { location: (0, 0), value: 5 }),
            (vec![0,53]               , TokNewLine { location: (0, 0) }),
            (vec![0,54]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,54,0]             , TokExpression),
            (vec![0,54,0,0]           , TokCompOp { location: (0, 0), op_name: "neq".to_string() }),
            (vec![0,54,0,0,0]         , TokInt { location: (0, 0), value: 12345 }),
            (vec![0,54,0,0,1]         , TokInt { location: (0, 0), value: 1234 }),
            (vec![0,55]               , TokNewLine { location: (0, 0) }),
            (vec![0,56]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,56,0]             , TokExpression),
            (vec![0,56,0,0]           , TokCompOp { location: (0, 0), op_name: "==".to_string() }),
            (vec![0,56,0,0,0]         , TokString { location: (0, 0), value: "hallo".to_string() }),
            (vec![0,56,0,0,1]         , TokString { location: (0, 0), value: "hallo".to_string() }),
            (vec![0,57]               , TokNewLine { location: (0, 0) }),
            (vec![0,58]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,58,0]             , TokExpression),
            (vec![0,58,0,0]           , TokCompOp { location: (0, 0), op_name: "==".to_string() }),
            (vec![0,58,0,0,0]         , TokString { location: (0, 0), value: "hallo".to_string() }),
            (vec![0,58,0,0,1]         , TokString { location: (0, 0), value: "hallo2".to_string() }),
            (vec![0,59]               , TokNewLine { location: (0, 0) }),
        );

        test_expected(expected, ast);
    }

    #[test]
    fn string_expressions_test() {
        let ast = test_ast("::Start\n<<print \"1234\">>\n<<print \"hallo\">>\n<<print 'hallo'>>\n<<print \"hal\"+'lo'>>\n<<print \"hal\"+\"lo\">>\n<<set $var to \"hallo\">>\n<<print $var>>\n<<print \"hallo \"+$var>>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (0, 0), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokString { location: (0, 0), value: "1234".to_string() }),
            (vec![0,1]                , TokNewLine { location: (0, 0) }),
            (vec![0,2]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokString { location: (0, 0), value: "hallo".to_string() }),
            (vec![0,3]                , TokNewLine { location: (0, 0) }),
            (vec![0,4]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokString { location: (0, 0), value: "hallo".to_string() }),
            (vec![0,5]                , TokNewLine { location: (0, 0) }),
            (vec![0,6]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,6,0,0,0]          , TokString { location: (0, 0), value: "hal".to_string() }),
            (vec![0,6,0,0,1]          , TokString { location: (0, 0), value: "lo".to_string() }),
            (vec![0,7]                , TokNewLine { location: (0, 0) }),
            (vec![0,8]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,8,0]              , TokExpression),
            (vec![0,8,0,0]            , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,8,0,0,0]          , TokString { location: (0, 0), value: "hal".to_string() }),
            (vec![0,8,0,0,1]          , TokString { location: (0, 0), value: "lo".to_string() }),
            (vec![0,9]                , TokNewLine { location: (0, 0) }),
            (vec![0,10]               , TokAssign { location: (0, 0), var_name: "$var".to_string(), op_name: "to".to_string() }),
            (vec![0,10,0]             , TokExpression),
            (vec![0,10,0,0]           , TokString { location: (0, 0), value: "hallo".to_string() }),
            (vec![0,11]               , TokNewLine { location: (0, 0) }),
            (vec![0,12]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,12,0]             , TokExpression),
            (vec![0,12,0,0]           , TokVariable { location: (0, 0), name: "$var".to_string() }),
            (vec![0,13]               , TokNewLine { location: (0, 0) }),
            (vec![0,14]               , TokMacroPrint { location: (0, 0) }),
            (vec![0,14,0]             , TokExpression),
            (vec![0,14,0,0]           , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,14,0,0,0]         , TokString { location: (0, 0), value: "hallo ".to_string() }),
            (vec![0,14,0,0,1]         , TokVariable { location: (0, 0), name: "$var".to_string() }),
            (vec![0,15]               , TokNewLine { location: (0, 0) }),
        );

        test_expected(expected, ast);
    }

    #[test]
    fn misc_expressions_test() {
        let ast = test_ast("::Start\n<<print random(1,100)+2>>\n<<print 5*3>7+3 and 5lte8>>\n<<print 15>10 or 4lte1>>\n<<if $var is 50>>fifty<<else if $var>50>>not fifty<<else>>not fifty!!<<endif>>\n");

        let expected = vec!(
            (vec![0]                  , TokPassage { location: (0, 0), name: "Start".to_string() }),
            (vec![0,0]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,0,0]              , TokExpression),
            (vec![0,0,0,0]            , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,0,0,0,0]          , TokFunction { location: (0, 0), name: "random".to_string() }),
            (vec![0,0,0,0,0,0]        , TokExpression),
            (vec![0,0,0,0,0,0,0]      , TokInt { location: (0, 0), value: 1 }),
            (vec![0,0,0,0,0,1]        , TokExpression),
            (vec![0,0,0,0,0,1,0]      , TokInt { location: (0, 0), value: 100 }),
            (vec![0,0,0,0,1]          , TokInt { location: (0, 0), value: 2 }),
            (vec![0,1]                , TokNewLine { location: (0, 0) }),
            (vec![0,2]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,2,0]              , TokExpression),
            (vec![0,2,0,0]            , TokLogOp { location: (0, 0), op_name: "and".to_string() }),
            (vec![0,2,0,0,0]          , TokCompOp { location: (0, 0), op_name: ">".to_string() }),
            (vec![0,2,0,0,0,0]        , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,2,0,0,0,0,0]      , TokInt { location: (0, 0), value: 5 }),
            (vec![0,2,0,0,0,0,1]      , TokInt { location: (0, 0), value: 3 }),
            (vec![0,2,0,0,0,1]        , TokNumOp { location: (0, 0), op_name: "+".to_string() }),
            (vec![0,2,0,0,0,1,0]      , TokInt { location: (0, 0), value: 7 }),
            (vec![0,2,0,0,0,1,1]      , TokInt { location: (0, 0), value: 3 }),
            (vec![0,2,0,0,1]          , TokCompOp { location: (0, 0), op_name: "lte".to_string() }),
            (vec![0,2,0,0,1,0]        , TokInt { location: (0, 0), value: 5 }),
            (vec![0,2,0,0,1,1]        , TokInt { location: (0, 0), value: 8 }),
            (vec![0,3]                , TokNewLine { location: (0, 0) }),
            (vec![0,4]                , TokMacroPrint { location: (0, 0) }),
            (vec![0,4,0]              , TokExpression),
            (vec![0,4,0,0]            , TokLogOp { location: (0, 0), op_name: "or".to_string() }),
            (vec![0,4,0,0,0]          , TokCompOp { location: (0, 0), op_name: ">".to_string() }),
            (vec![0,4,0,0,0,0]        , TokInt { location: (0, 0), value: 15 }),
            (vec![0,4,0,0,0,1]        , TokInt { location: (0, 0), value: 10 }),
            (vec![0,4,0,0,1]          , TokCompOp { location: (0, 0), op_name: "lte".to_string() }),
            (vec![0,4,0,0,1,0]        , TokInt { location: (0, 0), value: 4 }),
            (vec![0,4,0,0,1,1]        , TokInt { location: (0, 0), value: 1 }),
            (vec![0,5]                , TokNewLine { location: (0, 0) }),
            (vec![0,6]                , TokMacroIf { location: (0, 0) }),
            (vec![0,6,0]              , TokExpression),
            (vec![0,6,0,0]            , TokCompOp { location: (0, 0), op_name: "is".to_string() }),
            (vec![0,6,0,0,0]          , TokVariable { location: (0, 0), name: "$var".to_string() }),
            (vec![0,6,0,0,1]          , TokInt { location: (0, 0), value: 50 }),
            (vec![0,6,1]              , TokText { location: (0, 0), text: "fifty".to_string() }),
            (vec![0,7]                , TokMacroElseIf { location: (0, 0) }),
            (vec![0,7,0]              , TokExpression),
            (vec![0,7,0,0]            , TokCompOp { location: (0, 0), op_name: ">".to_string() }),
            (vec![0,7,0,0,0]          , TokVariable { location: (0, 0), name: "$var".to_string() }),
            (vec![0,7,0,0,1]          , TokInt { location: (0, 0), value: 50 }),
            (vec![0,7,1]              , TokText { location: (0, 0), text: "not fifty".to_string() }),
            (vec![0,8]                , TokMacroElse { location: (0, 0) }),
            (vec![0,8,0]              , TokText { location: (0, 0), text: "not fifty!!".to_string() }),
            (vec![0,9]                , TokMacroEndIf { location: (0, 0) }),
            (vec![0,10]               , TokNewLine { location: (0, 0) }),
        );

        test_expected(expected, ast);
    }
}
