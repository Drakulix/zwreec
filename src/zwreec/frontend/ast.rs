//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

pub use frontend::parser;
pub use frontend::lexer::Token;
pub use backend::zcode::zfile;

//==============================
// ast

pub struct AST {
    passages: Vec<ASTNode>
}

struct formatting_state{
    bold: bool,
    mono: bool,
    italic: bool,
    inverted: bool
}

/// add zcode based on tokens
fn stuff(node: &ASTNode,mut state: &Vec<formatting_state> , mut out: &mut zfile::Zfile) {
     match node {
        &ASTNode::Passage(ref t) => {
            match &t.category{
                _ => {
                    debug!("no match 1");
                }
            };
            for child in &t.childs {
                stuff(child, state, out);
            }
        },
        &ASTNode::Default(ref t) => {
            match &t.category{
                &Token::TokText(ref s) => {
                    out.op_print(s);
                },
                &Token::TokFormatBold => {
                    out.op_set_text_style(true, false, false, false);
                },
                 &Token::TokFormatItalic => {
                    out.op_set_text_style(false, false, false, true);
                },
                _ => {
                    debug!("no match 2");
                }
            };
            for child in &t.childs {
                stuff(child, state, out);
            }
            out.op_set_text_style(false, false, false, false);
        }
    };
}

impl AST {
    /// convert ast to zcode
    pub fn to_zcode(&self,  out: &mut zfile::Zfile){
        let mut state:Vec<formatting_state> = Vec::new();
        let base = formatting_state {bold: false, italic: false, mono: false, inverted: false};
        state.push(base);
        for child in &self.passages {
            stuff(child, &state, out);
        }
    }

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
        let node = ASTNode::Passage(NodePassage { category: token, childs: Vec::new() });
        self.passages.push(node);
    }

    /// adds a child to the path in the ast
    pub fn add_child(&mut self, path: &Vec<usize>, token: Token) {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);
            
            self.passages[*index].add_child(new_path, token)
        } else {
            self.passages.push(ASTNode::Default(NodeDefault { category: token, childs: Vec::new() }));
        }
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
}

// ================================
// node types
enum ASTNode {
    Default (NodeDefault),
    Passage (NodePassage)
}

struct NodePassage {
    category: Token,
    pub childs: Vec<ASTNode>,
    /*tags: Vec<ASTNode>*/
}

struct NodeDefault {
    category: Token,
    childs: Vec<ASTNode>
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

    /// prints an node of an ast
    pub fn print(&self, indent: usize) {
        let mut spaces = "".to_string();
        for _ in 0..indent { 
            spaces.push_str(" ");
        }

        match self {
            &ASTNode::Passage(ref t) => {
                debug!("{}|- : {:?}", spaces, t.category);
                for child in &t.childs {
                    child.print(indent+2);
                }
            },
            &ASTNode::Default(ref t) => {
                debug!("{}|- : {:?}", spaces, t.category);
                for child in &t.childs {
                    child.print(indent+2);
                }
            }
        }
    }
}


//==============================
// 

#[test]
fn it_works() {

}

