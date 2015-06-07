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
fn gen_zcode<'a>(node: &'a ASTNode, state: FormattingState, mut out: &mut zfile::Zfile, mut manager: &mut CodeGenManager<'a>) {
    let mut state_copy = state.clone();
    let mut set_formatting = false;
  
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
                gen_zcode(child, state_copy, out, manager);
            }

            out.op_newline();
            out.op_call_1n("system_check_links");

        },
        &ASTNode::Default(ref t) => {
            match &t.category {
                &Token::TokText(ref s) => {
                    out.gen_print_ops(s);
                },
                &Token::TokNewLine => {
                    out.op_newline();
                },
                &Token::TokFormatBoldStart => {
                    state_copy.bold = true;
                    out.op_set_text_style(state_copy.bold, state_copy.inverted, state_copy.mono, state_copy.italic);
                    set_formatting = true;
                },
                &Token::TokFormatItalicStart => {
                    state_copy.italic = true;
                    out.op_set_text_style(state_copy.bold, state_copy.inverted, state_copy.mono, state_copy.italic);
                    set_formatting = true;
                },
                &Token::TokPassageLink (ref name, ref link) => {
                    out.op_call_2n_with_address("system_add_link", link);

                    out.op_set_text_style(state_copy.bold, true, state_copy.mono, state_copy.italic);
                    let link_text = format!("{}[", name);
                    out.op_print(&link_text);
                    out.op_print_num_var(16);
                    out.op_print("]");
                    out.op_set_text_style(state_copy.bold, state_copy.inverted, state_copy.mono, state_copy.italic);
                    set_formatting = true;
                },
                &Token::TokAssign(ref var, ref operator) => {
                    if operator == "=" || operator == "to" {
                        if !manager.symbol_table.is_known_symbol(var) {
                            manager.symbol_table.insert_new_symbol(&var);
                        }
                        let symbol_id = manager.symbol_table.get_symbol_id(var);
                        if t.childs.len() == 1 {
                            match t.childs[0].as_default().category {
                                Token::TokInt(value) => {
                                    out.op_store_u16(symbol_id, value as u16);
                                },
                                Token::TokBoolean(ref bool_val) => {
                                    out.op_store_u8(symbol_id, boolstr_to_u8(&*bool_val));
                                }
                                _ => { }
                            }
                        } else {
                            debug!("Assign Expression currently not supported.");
                        }
                    }
                },
                &Token::TokIf => {
                    if t.childs.len() < 2 {
                        panic!("Unsupported if-expression!");
                    }

                    let mut compare: u8 = 1;

                    // check if the first node is a pseudonode
                    let pseudo_node = match t.childs[0].as_default().category {
                        Token::TokPseudo => t.childs[0].as_default(),
                        _ =>  panic!("Unsupported if-expression!")
                    };

                    // Check if first token is variable
                    let var_name = match pseudo_node.childs[0].as_default().category {
                        Token::TokVariable(ref var) => var,
                        _ =>  panic!("Unsupported if-expression!")
                    };

                    if pseudo_node.childs.len() > 1 {
                        // Check if second token is compare operator
                        match pseudo_node.childs[1].as_default().category {
                            Token::TokCompOp(ref op) => {
                                match &*(*op) {
                                    "==" | "is" => {} ,
                                    _ => panic!("Unsupported Compare Operator!")
                                }
                            }, _ =>  panic!("Unsupported if-expression!")
                        }

                        // Check if third token is number
                        compare = match pseudo_node.childs[2].as_default().category {
                            Token::TokInt(ref value) => {
                                *value as u8
                            },
                            Token::TokBoolean(ref bool_val) => {
                                boolstr_to_u8(&*bool_val)
                            }, _ => panic!("Unsupported assign value!") 
                        };
                    }

                    let symbol_id = manager.symbol_table.get_symbol_id(&*var_name);
                    let if_id = manager.ids_if.start_next();

                    let if_label = &format!("if_{}", if_id);
                    let after_if_label = &format!("after_if_{}", if_id);
                    let after_else_label = &format!("after_else_{}", if_id);
                    out.op_je(symbol_id, compare, if_label);
                    out.op_jump(after_if_label);
                    out.label(if_label);

                    for i in 1..t.childs.len() {
                        gen_zcode(&t.childs[i], state_copy, out, manager)
                    }

                    out.op_jump(after_else_label);
                    out.label(after_if_label);
                },
                &Token::TokElse => {
                    for child in &t.childs {
                        gen_zcode(child, state_copy, out, manager)
                    }
                },
                &Token::TokEndIf => {
                    let after_else_label = &format!("after_else_{}", manager.ids_if.pop_id());
                    out.label(after_else_label);
                },
                _ => {
                    debug!("no match 2");
                }
            };
            if set_formatting {
                for child in &t.childs {
                    gen_zcode(child, state_copy, out, manager);
                }
                out.op_set_text_style(false, false, false, false);
                out.op_set_text_style(state.bold, state.inverted, state.mono, state.italic);
            }
        }
    };

   
}

fn boolstr_to_u8(string: &str) -> u8 {
    match string {
        "true" => 1 as u8,
        _ => 0 as u8
    }
}

impl AST {
    /// convert ast to zcode
    pub fn to_zcode(& self, out: &mut zfile::Zfile) {
        let mut manager = CodeGenManager::new();
        let state = FormattingState {bold: false, italic: false, mono: false, inverted: false};
        for child in &self.passages {
            gen_zcode(child, state, out, &mut manager);
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

struct CodeGenManager<'a> {
    ids_if: IdentifierProvider,
    symbol_table: SymbolTable<'a>
}

struct IdentifierProvider {
    current_id: u32,
    id_stack: Vec<u32>
}

struct SymbolTable<'a> {
    current_id: u8,
    symbol_map: HashMap<&'a str, u8>
}

impl <'a> CodeGenManager<'a> {
    pub fn new() -> CodeGenManager<'a> {
        CodeGenManager {
            ids_if: IdentifierProvider::new(),
            symbol_table: SymbolTable::new()
        }
    }
}

impl IdentifierProvider {
    pub fn new() -> IdentifierProvider {
        IdentifierProvider {
            current_id: 0, 
            id_stack: Vec::new()
        }
    }

    // Returns a new id and pushes it onto the stack
    pub fn start_next(&mut self) -> u32 {
        let id = self.current_id;
        self.current_id += 1;
        self.id_stack.push(id);
        id
    }

    // Pops the last id from the stack
    pub fn pop_id(&mut self) -> u32 {
        self.id_stack.pop().unwrap()
    }
}

impl <'a> SymbolTable<'a> {
    pub fn new() -> SymbolTable<'a> {
        SymbolTable {
            current_id: 25,
            symbol_map: HashMap::<&str, u8>::new()
        }
    }

    // Inserts a symbol into the table, assigning a new id
    pub fn insert_new_symbol(&mut self, symbol: &'a str) {
        debug!("Assigned id {} to variable {}", self.current_id, symbol);
        self.symbol_map.insert(symbol, self.current_id);
        self.current_id += 1;
    }

    // Checks if the symbol is already existent in the table
    pub fn is_known_symbol(&self, symbol: &str) -> bool {
        self.symbol_map.contains_key(symbol)
    }

    // Returns the id for a given symbol 
    // (check if is_known_symbol, otherwise panics)
    pub fn get_symbol_id(&self, symbol: &str) -> u8 {
        *self.symbol_map.get(symbol).unwrap()
    }
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

    pub fn as_default(&self) -> &NodeDefault {
        match self { 
            &ASTNode::Default(ref def) => def, 
            _ => panic!("Node cannot be unwrapped as NodeDefault!")
        }
    }
}
