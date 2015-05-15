//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

pub use frontend::parser;
pub use frontend::lexer::Token;


//==============================
// ast

pub struct AST {
    passages: Vec<NodeType>
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
        debug!("");
    }

    /// adds a passage to the path in the ast
    pub fn add_passage(&mut self, token: Token) {
        let node = NodeType::Passage(NodePassage { category: token, content: Vec::new() });
        self.passages.push(node);
    }

    /// adds a leaf to the path in the ast
    pub fn add_leaf(&mut self, path: &Vec<usize>, token: Token) {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);
            
            self.passages[*index].add_leaf(new_path, token)
        } else {
            panic!("leaf should add at the tree-path")
        }
    }

    /// adds a child to the path in the ast
    pub fn add_child(&mut self, path: &Vec<usize>, token: Token) {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);
            
            self.passages[*index].add_child(new_path, token)
        } else {
            self.passages.push(NodeType::Default(NodeDefault { category: token, childs: Vec::new() }));
        }
    }

    /// counts the childs of the path in the asts
    pub fn cound_childs(&self, path: Vec<usize>) -> usize {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);
            
            self.passages[*index].cound_childs(new_path)
        } else {
            self.passages.len()
        }
    }

}

// ================================
// node types
enum NodeType {
    Default (NodeDefault),
    Passage (NodePassage),
    Leaf (NodeLeaf)
}

struct NodePassage {
    category: Token,
    pub content: Vec<NodeType>,
    /*tags: Vec<NodeType>*/
}

struct NodeDefault {
    category: Token,
    childs: Vec<NodeType>
}

struct NodeLeaf {
    category: Token
}

impl NodeType {
    /// adds an leaf to the path in the ast
    pub fn add_leaf(&mut self, path: Vec<usize>, token: Token) {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            match self {
                &mut NodeType::Default(ref mut node) => node.childs[*index].add_leaf(new_path, token),
                &mut NodeType::Passage(ref mut node) => node.content[*index].add_leaf(new_path, token),
                &mut NodeType::Leaf(_) => { panic!("error, leaf has no other leaf") }
            }
        } else {
            match self {
                &mut NodeType::Default(ref mut node) => node.childs.push(NodeType::Leaf(NodeLeaf { category: token } )),
                &mut NodeType::Passage(ref mut node) => node.content.push(NodeType::Leaf(NodeLeaf { category: token } )),
                &mut NodeType::Leaf(_) => { panic!("error, leaf has no other leaf") }
            }
        }
    }

    /// adds an child to the path in the ast
    pub fn add_child(&mut self, path: Vec<usize>, token: Token) {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            match self {
                &mut NodeType::Default(ref mut node) => node.childs[*index].add_child(new_path, token),
                &mut NodeType::Passage(ref mut node) => node.content.push(NodeType::Default(NodeDefault { category: token, childs: Vec::new() } )),
                &mut NodeType::Leaf(_) => { panic!("error, leaf has no childs") }
            }
        } else {
            match self {
                &mut NodeType::Default(ref mut node) => node.childs.push(NodeType::Default(NodeDefault { category: token, childs: Vec::new() } )),
                &mut NodeType::Passage(ref mut node) => node.content.push(NodeType::Default(NodeDefault { category: token, childs: Vec::new() } )),
                &mut NodeType::Leaf(_) => { panic!("error, leaf has no childs") }
            }
        }
    }

    /// counts the childs of the current path in the ast
    pub fn cound_childs(&self, path: Vec<usize>) -> usize {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            match self {
                &NodeType::Default(ref node) => node.childs[*index].cound_childs(new_path),
                &NodeType::Passage(ref node) => node.content[*index].cound_childs(new_path),
                &NodeType::Leaf(_) => { panic!("error, leaf has no childs") }
            }
        } else {
            match self {
                &NodeType::Default(ref node) => node.childs.len(),
                &NodeType::Passage(ref node) => node.content.len(),
                &NodeType::Leaf(_) => { panic!("error, leaf has no childs") }
            }
        }
    }

    /// prints an node of an ast
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
            &NodeType::Default(ref t) => {
                debug!("{}|- : {:?}", spaces, t.category);
                for child in &t.childs {
                    child.print(indent+2);
                }
            },
            &NodeType::Leaf(ref t) => { debug!("{}|- {:?}", spaces, t.category); }
        }
    }
}


//==============================
// 

#[test]
fn it_works() {

}

