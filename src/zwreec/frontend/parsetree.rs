//! The `parsetree` module contains a lot of useful functionality
//! to create and walk through the parsetree

use frontend::lexer::Token;
use frontend::parser::{NonTerminalType};

//==============================
// parsetree

pub struct ParseTree {
    root: PNode
}

impl ParseTree {
    pub fn new() -> ParseTree {
        ParseTree {
            root: PNode::new_non_terminal(NonTerminalType::S)
        }
    }

    /// adds nodes to the tree
    pub fn add_nodes(&mut self, childs: Vec<PNode>, to_add_path: &Vec<usize>) {
        for child in childs {
            self.root.add_child_at(to_add_path, child);
        }
    }

    /// prints the tree
    pub fn print(&self) {
        debug!("Parse Tree:");
        self.root.print(0);
        debug!("");
    }

    /// checks if
    pub fn is_terminal(&self, path: Vec<usize>) -> bool {
        self.root.is_terminal(path)
    }

    pub fn get_non_terminal(&self, path: Vec<usize>) -> &NonTerminalType {
        self.root.get_non_terminal(path)
    }
}

//==============================
// node of the paretree

pub struct NodeNonTerminal {
    category: NonTerminalType,
    childs: Vec<PNode>
}

pub struct NodeTerminal {
    category: Token
}

//#[derive(Debug)]
pub enum PNode {
    NonTerminal (NodeNonTerminal),
    Terminal (NodeTerminal)
}

impl PNode {
    pub fn new_terminal(terminal: Token) -> PNode {
        PNode::Terminal(NodeTerminal  { category: terminal })
    }

    pub fn new_non_terminal(non_terminal: NonTerminalType) -> PNode {
        PNode::NonTerminal(NodeNonTerminal { category: non_terminal, childs: Vec::new() })
    }

    /// prints a node
    pub fn print(&self, indent: usize) {
        let mut spaces = "".to_string();
        for _ in 0..indent {
            spaces.push_str(" ");
        }

        match self {
            &PNode::NonTerminal(ref t) => {
                debug!("{}|- Node: {:?}", spaces, t.category);
                for child in &t.childs {
                    child.print(indent+2);
                }
            }
            &PNode::Terminal(ref t) => { debug!("{}|- Terminal: {:?}", spaces, t.category); }
        }
    }

    /// returns a non_terminal of the path
    pub fn get_non_terminal(&self, path: Vec<usize>) -> &NonTerminalType {
        match self {
            &PNode::NonTerminal(ref node) => {
                if let Some(index) = path.first() {
                    let mut new_path: Vec<usize> = path.to_vec();
                    new_path.remove(0);
                    return node.childs[*index].get_non_terminal(new_path);
                } else {
                    return &node.category;
                }
            },
            _ => panic!("error")
        }
    }

    /// adds a node to the childs in path
    pub fn add_child_at(&mut self, path: &[usize], child: PNode) {
        match self {
            &mut PNode::NonTerminal (ref mut node) => {
                if let Some(index) = path.first() {
                    let mut new_path: Vec<usize> = path.to_vec();
                    new_path.remove(0);
                    node.childs[*index].add_child_at(&new_path, child);
                } else {
                    node.childs.push(child);
                }
            },
            _ => panic!("error")
        }
    }

    /// checks if the node of the path is a terminal or not
    pub fn is_terminal(&self, path: Vec<usize>) -> bool {
        match self {
            &PNode::NonTerminal(ref node) => {
                if let Some(index) = path.first() {
                    let mut new_path: Vec<usize> = path.to_vec();
                    new_path.remove(0);
                    return node.childs[*index].is_terminal(new_path);
                }

                return false
            },
            &PNode::Terminal(_) => return true
        }

        false
    }
}
