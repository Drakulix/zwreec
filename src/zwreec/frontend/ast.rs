//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

use frontend::lexer::Token;
use backend::zcode::zfile;
use backend::zcode::zfile::{FormattingState, ZOP};
use std::collections::HashMap;

//==============================
// ast

pub struct AST {
    passages: Vec<ASTNode>
}



/// add zcode based on tokens
fn gen_zcode<'a>(node: &'a ASTNode, state: FormattingState, mut out: &mut zfile::Zfile, mut var_table: &mut HashMap<&'a str, u8>, mut var_id: &mut u8) -> Vec<ZOP> {
    let mut state_copy = state.clone();
    match node {
        &ASTNode::Passage(ref node) => {
            let mut code: Vec<ZOP> = vec![];
            match &node.category {
                &Token::TokPassageName(ref name) => {
                    code.push(ZOP::Routine{name: name.to_string(), count_variables: 0});
                },
                _ => {
                    debug!("no match 1");
                }
            };
            
            for child in &node.childs {
                for instr in gen_zcode(child, state_copy, out, var_table, var_id) {
                    code.push(instr);
                }

            }

            code.push(ZOP::Newline);
            code.push(ZOP::Call{jump_to_label: "system_check_links".to_string()});
            code
        },
        &ASTNode::Default(ref t) => {
            let mut code: Vec<ZOP> = match &t.category {
                &Token::TokText(ref s) => {
                    vec![ZOP::PrintOps{text: s.to_string()}]
                },
                &Token::TokNewLine => {
                    vec![ZOP::Newline]
                },
                &Token::TokFormatBoldStart => {
                    state_copy.bold = true;
                    vec![ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic}]
                },
                &Token::TokFormatItalicStart => {
                    state_copy.italic = true;
                    vec![ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic}]
                },
                &Token::TokPassageLink (ref name, ref link) => {
                    vec![
                    ZOP::CallWithAddress{jump_to_label: "system_add_link".to_string(), address: link.to_string()},
                    ZOP::SetTextStyle{bold: state_copy.bold, reverse: true, monospace: state_copy.mono, italic: state_copy.italic},
                    ZOP::Print{text: format!("{}[", name)},
                    ZOP::PrintNumVar{variable: 16},
                    ZOP::Print{text: "]".to_string()},
                    ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic}
                    ]
                },
                &Token::TokAssign(ref var, ref operator) => {
                    if operator == "=" || operator == "to" {
                        if !var_table.contains_key::<str>(var) {
                            var_table.insert(&var, *var_id);
                            debug!("Assigned id {} to variable {}", var_id, var);
                            *var_id += 1;
                        }
                        let id_option = var_table.get::<str>(var);
                        if t.childs.len() == 1 {
                            match t.childs[0] {
                                ASTNode::Default(ref def) => {
                                    let actual_id :u8 = match id_option {
                                        Some(id) => {
                                            *id                                             
                                        },
                                        None => {
                                            panic!("Variable not in var table.")
                                        }
                                    };
                                    match def.category {
                                        Token::TokInt(value) => {
                                            vec![
                                            ZOP::StoreU16{variable: actual_id, value: value as u16},
                                            ZOP::PrintNumVar{variable: actual_id}
                                            ]
                                        },
                                        Token::TokBoolean(ref bool_val) => {
                                            let value = match (*bool_val).as_ref() {
                                                "true" => { 1 as u8 },
                                                _ => { 0 as u8 }
                                            };
                                            vec![ZOP::StoreU8{variable: actual_id, value: value}]
                                        }
                                        _ => { vec![] }
                                    }
                                },
                                _ => { vec![] }
                            }
                        } else {
                            debug!("Assign Expression currently not supported.");
                            vec![]
                        }
                        
                    } else { vec![] }
                },
                _ => {
                    debug!("no match 2");
                    vec![]
                }
            };

            for child in &t.childs {
                for instr in gen_zcode(child, state_copy, out, var_table, var_id) {
                    code.push(instr);
                }
            }

            code.push(ZOP::SetTextStyle{bold: false, reverse: false, monospace: false, italic: false});
            code.push(ZOP::SetTextStyle{bold: state.bold, reverse: state.inverted, monospace: state.mono, italic: state.italic});
            code
        }
    }
}

impl AST {
    /// convert ast to zcode
    pub fn to_zcode(&self,  out: &mut zfile::Zfile) {
        let mut var_table = HashMap::<&str, u8>::new();
        let mut var_id : u8 = 25;
        let state = FormattingState {bold: false, italic: false, mono: false, inverted: false};
        let mut code : Vec<ZOP> = vec![];
        for child in &self.passages {
            for instr in gen_zcode(child, state, out, &mut var_table, &mut var_id) {
                code.push(instr);
            }
        }
        debug!("emit zcode:");
        for instr in &code {
            debug!("{:?}", instr);
        }
        out.emit(code);
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
