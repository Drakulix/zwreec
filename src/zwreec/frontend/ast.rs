//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

use backend::zcode::zfile;
use backend::zcode::zfile::{ZOP, Type};
use frontend::codegen;
use frontend::expressionparser;
use frontend::lexer::Token;
use frontend::lexer::Token::{TokMacroIf, TokMacroElseIf, TokExpression, TokPassage};

//==============================
// ast

pub struct AST {
    passages: Vec<ASTNode>,
    path: Vec<usize>,
    is_in_if_expression: bool
}

pub enum ASTOperation {
    AddPassage(Token),
    AddChild(Token),
    ChildDown(Token),
    Up,
    UpChild(Token),
    UpChildDown(Token),
    UpSpecial,
}

impl AST {
    pub fn build<I: Iterator<Item=ASTOperation>>(ops: I) -> AST {
        let mut ast = AST {
            passages: Vec::new(),
            path: Vec::new(),
            is_in_if_expression: false,
        };
        for op in ops {
            ast.operation(op);
        }
        ast.parse_expressions();

        ast
    }

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
        for child in &mut self.passages {
            child.parse_expressions();
        }
    }

    /// adds a passage to the path in the ast
    pub fn add_passage(&mut self, token: Token) {
        self.path.clear();
        let ast_count_passages = self.count_childs(self.path.to_vec());

        let node = ASTNode::Passage(NodePassage { category: token, childs: Vec::new() });
        self.passages.push(node);

        self.path.push(ast_count_passages);
    }

    /// adds a child to the path in the ast
    pub fn add_child(&mut self, token: Token) {
        if let Some(index) = self.path.first() {
            let mut new_path: Vec<usize> = self.path.to_vec();
            new_path.remove(0);
            self.passages[*index].add_child(new_path, token);
        } else {
            self.passages.push(ASTNode::Default(NodeDefault { category: token, childs: Vec::new() }));
        }
    }

    /// adds a child an goees one child down
    pub fn child_down(&mut self, token: Token) {
        //
        if token.clone() == (TokMacroIf { location: (0, 0) }) ||
           token.clone() == (TokMacroElseIf { location: (0, 0) }) {
            self.is_in_if_expression = true;
        }

        let ast_count_childs = self.count_childs(self.path.to_vec());
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

    /// prints the tree
    pub fn print(&self, force_print: bool) {
        debug!("Abstract Syntax Tree: ");
        for child in &self.passages {
            child.print(0, force_print);
        }
        debug!("");
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
                    token == node.category
                },
                &ASTNode::Passage(ref node) => {
                    token == node.category
                },
            }
        }
    }

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

    /// prints an node of an ast
    pub fn print(&self, indent: usize, force_print: bool) {
        let mut spaces = "".to_string();
        for _ in 0..indent {
            spaces.push_str(" ");
        }

        match self {
            &ASTNode::Passage(ref t) => {
                if force_print {
                    println!("{}|- : {:?}", spaces, t.category);
                } else {
                    debug!("{}|- : {:?}", spaces, t.category);
                }
                for child in &t.childs {
                    child.print(indent+2, force_print);
                }
            },
            &ASTNode::Default(ref t) => {
                if force_print {
                    println!("{}|- : {:?}", spaces, t.category);
                } else {
                    debug!("{}|- : {:?}", spaces, t.category);
                }
                for child in &t.childs {
                    child.print(indent+2, force_print);
                }
            }
        }
    }

    pub fn as_default(&self) -> &NodeDefault {
        match self {
            &ASTNode::Default(ref def) => def,
            _ => panic!("Node cannot be unwrapped as NodeDefault!")
        }
    }

    /// goes through the whole tree and parse the expressions
    fn parse_expressions(&mut self) {
        match self {
            &mut ASTNode::Passage(ref mut node) => {
                for mut child in node.childs.iter_mut() {
                    child.parse_expressions();
                }
            },
            &mut ASTNode::Default(ref mut node) => {
                match &node.category {
                    &TokExpression => {
                        expressionparser::ExpressionParser::parse(node);
                    },
                    _ => ()
                }

                for mut child in node.childs.iter_mut() {
                    child.parse_expressions();
                }
            }
        }
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
        AST::build(parser.parse(tokens.inspect(|ref token| {
            println!("{:?}", token);
        })))
    }

    /// checks expected
    fn test_expected(expected: Vec<(Vec<usize>, Token)>, ast: AST) {
        for item in expected.iter() {
            let b = ast.is_specific_token(item.1.clone(), item.0.to_vec());
            if b == false {
                ast.print(true);
            }
            assert!(ast.is_specific_token(item.1.clone(), item.0.to_vec()));
        }
    }

    #[test]
    fn text_test() {
        let ast = test_ast("::Passage\nTestText\nTestNextLine\n::NextPassage");

        let expected = vec!(
            (vec![0]  , TokPassage {name: "Passage".to_string(), location: (0, 0)}),
            (vec![0,0], TokText {location: (0, 0), text: "TestText".to_string()}),
            (vec![0,1], TokNewLine {location: (0, 0)} ),
            (vec![0,2], TokText {location: (0, 0), text: "".to_string()}),
            (vec![0,1], TokNewLine {location: (0, 0)}),
            (vec![1]  , TokPassage {name: "".to_string(), location: (0, 0)}),

        );

        test_expected(expected, ast);
    }

    #[test]
    fn test_num_expressions() {
        let ast = test_ast("::Start\n<<print -12345>>\n<<print 5>>\n<<print 32767>>\n<<print 1*2*3*4*5*6*7>>\n<<print 1*2+3*4+5*6+7>>\n<<print 1*2-3*4-5*6-7>>\n<<print 256/8/4/8>>\n");

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

        );

        test_expected(expected, ast);
    }
}
