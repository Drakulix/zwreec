//! ast (abstract syntaxtree)
//! ...

pub use frontend::parser;
pub use frontend::lexer::Token;

pub fn create_ast(ptree: &parser::ParseTree) {
    let mut ast: AST = AST::new();
    ptree.create_ast(&mut ast);

    ast.print();
}


//==============================
// ast

pub struct AST {
    passages: Vec<NodeType>
}

enum NodeType {
    /*Default (NodeDefault),*/
    Passage (NodePassage),
    Leaf (NodeLeaf)
}

// ================================
// node types

struct NodePassage {
    category: Token,
    pub content: Vec<NodeType>,
    /*tags: Vec<NodeType>*/
}

/*struct NodeDefault {
    category: Token,
    childs: Vec<NodeType>
}*/

struct NodeLeaf {
    category: Token
}

impl AST {
    pub fn new() -> AST {
        AST {
            passages: Vec::new()
        }
    }

    /// prints the tree
    pub fn print(&self) {
        debug!("Abstract Syntax Tree: ");
        for child in &self.passages {
            child.print(0);
        }
    }

    pub fn add_passage(&mut self, token: Token) {
        let node = NodeType::Passage(NodePassage { category: token, content: Vec::new()/*, tags: Vec::new()*/ });
        self.passages.push(node);
    }

    pub fn add_leaf(&mut self, token: Token) {
        if let Some(mut last_passage) = self.passages.last_mut() {
            match last_passage {
                &mut NodeType::Passage(ref mut p) => {
                    let leaf = NodeType::Leaf(NodeLeaf { category: token });
                    p.content.push(leaf);
                },
                _ => {}
            }
        }
    }
}

impl NodeType {
    pub fn print(&self, indent: usize) {
        let mut spaces = "".to_string();
        for _ in 0..indent { 
            spaces.push_str(" ");
        }

        match self {
            &NodeType::Passage(ref t) => {
                debug!("{}|- : {:?}", spaces, t.category);
                for child in &t.content {
                    child.print(indent+2);
                }
            },
            /*&NodeType::Default(ref t) => {
                debug!("{}|- Node: {:?}", spaces, t.category);
                for child in &t.childs {
                    child.print(indent+2);
                }
            },*/
            &NodeType::Leaf(ref t) => { debug!("{}|- {:?}", spaces, t.category); }
        }
    }
}


//==============================
// 

#[test]
fn it_works() {

}

