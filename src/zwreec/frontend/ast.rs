//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

use frontend::lexer::Token;
use backend::zcode::zfile;
use backend::zcode::zfile::{FormattingState};
use std::collections::HashMap;

//==============================
// ast

pub struct AST {
    passages: Vec<ASTNode>
}



/// add zcode based on tokens
fn gen_zcode<'a>(node: &'a ASTNode, state: FormattingState, mut out: &mut zfile::Zfile, mut var_table: &mut HashMap<&'a str, u8>, mut var_id: &mut u8) {
    let mut state_copy = state.clone();
  
    match node {
        &ASTNode::Passage(ref node) => {
            match &node.category {
                &Token::TokPassageName(ref name) => {
                    out.routine(name, 0);
                },
                _ => {
                    debug!("no match 1");
                }
            };
            
            for child in &node.childs {
                gen_zcode(child, state_copy, out, var_table, var_id);
            }

            out.op_newline();
            out.op_call_1n("system_check_links");

        },
        &ASTNode::Default(ref t) => {
            match &t.category {
                &Token::TokText(ref s) => {
                    out.op_print(s);
                },
                &Token::TokNewLine => {
                    out.op_newline();
                },
                &Token::TokFormatBoldStart => {
                    state_copy.bold = true;
                    out.op_set_text_style(state_copy.bold, state_copy.inverted, state_copy.mono, state_copy.italic);
                },
                &Token::TokFormatItalicStart => {
                    state_copy.italic = true;
                    out.op_set_text_style(state_copy.bold, state_copy.inverted, state_copy.mono, state_copy.italic);
                },
                &Token::TokPassageLink (ref name, ref link) => {
                    out.op_call_2n_with_address("system_add_link", link);

                    out.op_set_text_style(state_copy.bold, true, state_copy.mono, state_copy.italic);
                    let link_text = format!("{}[", name);
                    out.op_print(&link_text);
                    out.op_print_num_var(16);
                    out.op_print("]");
                    out.op_set_text_style(state_copy.bold, state_copy.inverted, state_copy.mono, state_copy.italic);
                },
                &Token::TokAssign(ref var, ref operator) => {
                    if operator == "=" {
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
                                            out.op_store_u8(actual_id, value as u8);
                                        },
                                        Token::TokBoolean(ref bool_val) => {
                                            let value = match (*bool_val).as_ref() {
                                                "true" => { 1 as u8 },
                                                _ => { 0 as u8 }
                                            };
                                            out.op_store_u8(actual_id, value);
                                        }
                                        _ => { }
                                    }
                                },
                                _ => { }
                            }
                        } else {
                            debug!("Assign Expression currently not supported.");
                        }
                        
                    }
                },
                _ => {
                    debug!("no match 2");
                }
            };

            for child in &t.childs {
                gen_zcode(child, state_copy, out, var_table, var_id);
            }

            out.op_set_text_style(false, false, false, false);
            out.op_set_text_style(state.bold, state.inverted, state.mono, state.italic);
        }
    };
}

impl AST {
    /// convert ast to zcode
    pub fn to_zcode(&self,  out: &mut zfile::Zfile) {
        let mut var_table = HashMap::<&str, u8>::new();
        let mut var_id : u8 = 100;
        let state = FormattingState {bold: false, italic: false, mono: false, inverted: false};
        for child in &self.passages {
            gen_zcode(child, state, out, &mut var_table, &mut var_id);
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
