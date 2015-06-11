//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

use frontend::lexer::Token;
use backend::zcode::zfile;
use backend::zcode::zfile::{FormattingState, ZOP};
use std::collections::HashMap;

//==============================
// ast
#[derive(Clone)]
enum Type{
    Bool,
    Integer,
    String,
}

pub struct AST {
    passages: Vec<ASTNode>
}

 /// add zcode based on tokens
fn gen_zcode<'a>(node: &'a ASTNode, mut out: &mut zfile::Zfile, mut manager: &mut CodeGenManager<'a>) -> Vec<ZOP> {
    let mut state_copy = manager.format_state.clone();
    let mut set_formatting = false;
  
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
                for instr in gen_zcode(child, out, manager) {
                    code.push(instr);
                }

            }

            code.push(ZOP::Newline);
            code.push(ZOP::Call1N{jump_to_label: "system_check_links".to_string()});
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
                    set_formatting = true;
                    vec![ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic}]
                },
                &Token::TokFormatItalicStart => {
                    state_copy.italic = true;
                    set_formatting = true;
                    vec![ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic}]
                },
                &Token::TokPassageLink (ref name, ref link) => {
                    set_formatting = true;
                    vec![
                    ZOP::Call2NWithAddress{jump_to_label: "system_add_link".to_string(), address: link.to_string()},
                    ZOP::SetColor{foreground: 8, background: 2},
                    ZOP::Print{text: format!("{}[", name)},
                    ZOP::PrintNumVar{variable: 16},
                    ZOP::Print{text: "]".to_string()},
                    ZOP::SetColor{foreground: 9, background: 2},
                    ]
                },
                &Token::TokAssign(ref var, ref operator) => {
                    if operator == "=" || operator == "to" {
                        if t.childs.len() == 1 {
                            match t.childs[0].as_default().category {
                                Token::TokInt(value) => {
                                    if !manager.symbol_table.is_known_symbol(var) {
                                        manager.symbol_table.insert_new_symbol(&var, Type::Integer);
                                    }
                                    let symbol_id = manager.symbol_table.get_symbol_id(var);
                                    vec![ZOP::StoreU16{variable: symbol_id, value: value as u16}]
                                },
                                Token::TokBoolean(ref bool_val) => {
                                    if !manager.symbol_table.is_known_symbol(var) {
                                        manager.symbol_table.insert_new_symbol(&var, Type::Bool);
                                    }
                                    let symbol_id = manager.symbol_table.get_symbol_id(var);
                                    vec![ZOP::StoreU8{variable: symbol_id, value: boolstr_to_u8(&*bool_val)}]
                                },
                                _ => { vec![] }
                            }
                        } else {
                            debug!("Assign Expression currently not supported.");
                            vec![]
                        }
                    } else { vec![] }
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

                    let if_label = format!("if_{}", if_id);
                    let after_if_label = format!("after_if_{}", if_id);
                    let after_else_label = format!("after_else_{}", if_id);
                    let mut code: Vec<ZOP> = vec![
                        ZOP::JE{local_var_id: symbol_id, equal_to_const: compare, jump_to_label: if_label.to_string()},
                        ZOP::Jump{jump_to_label: after_if_label.to_string()},
                        ZOP::Label{name: if_label.to_string()}
                    ];

                    for i in 1..t.childs.len() {
                        for instr in gen_zcode(&t.childs[i], out, manager) {
                            code.push(instr);
                        }
                    }

                    code.push(ZOP::Jump{jump_to_label: after_else_label});
                    code.push(ZOP::Label{name: after_if_label});
                    code
                },
                &Token::TokElse => {
                    let mut code: Vec<ZOP> = vec![];
                    for child in &t.childs {
                        for instr in gen_zcode(child, out, manager) {
                            code.push(instr);
                        }
                    }
                    code
                },
                &Token::TokEndIf => {
                    let after_else_label = format!("after_else_{}", manager.ids_if.pop_id());
                    vec![ZOP::Label{name: after_else_label}]
                },
                &Token::TokMakroVar(ref name) => {
                    let var_id = manager.symbol_table.get_symbol_id(&*name);
                    match manager.symbol_table.get_symbol_type(&*name) {
                        Type::Integer => {
                            vec![ZOP::PrintNumVar{variable: var_id}]
                        },
                        Type::String => {
                            vec![]
                        },
                        Type::Bool => {
                            vec![ZOP::PrintNumVar{variable: var_id}]
                        }
                    }
                },
                _ => {
                    debug!("no match if");
                    vec![]
                }
            };
            if set_formatting {
                for child in &t.childs {
                    for instr in gen_zcode(child, out, manager) {
                        code.push(instr);
                    }
                }
                code.push(ZOP::SetTextStyle{bold: false, reverse: false, monospace: false, italic: false});
                let state = manager.format_state;
                code.push(ZOP::SetTextStyle{bold: state.bold, reverse: state.inverted, monospace: state.mono, italic: state.italic});
            }
            code
        }
    }

   
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
        let mut code: Vec<ZOP> = vec![];
        for child in &self.passages {
            for instr in gen_zcode(child, out, &mut manager) {
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

struct CodeGenManager<'a> {
    ids_if: IdentifierProvider,
    symbol_table: SymbolTable<'a>,
    format_state: FormattingState
}

struct IdentifierProvider {
    current_id: u32,
    id_stack: Vec<u32>
}

struct SymbolTable<'a> {
    current_id: u8,
    symbol_map: HashMap<&'a str, (u8, Type)>
}

impl <'a> CodeGenManager<'a> {
    pub fn new() -> CodeGenManager<'a> {
        CodeGenManager {
            ids_if: IdentifierProvider::new(),
            symbol_table: SymbolTable::new(),
            format_state: FormattingState {bold: false, italic: false, mono: false, inverted: false}
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
            symbol_map: HashMap::<&str, (u8,Type)>::new()
        }
    }

    // Inserts a symbol into the table, assigning a new id
    pub fn insert_new_symbol(&mut self, symbol: &'a str, t: Type) {
        debug!("Assigned id {} to variable {}", self.current_id, symbol);
        self.symbol_map.insert(symbol, (self.current_id,t));
        self.current_id += 1;
    }

    // Checks if the symbol is already existent in the table
    pub fn is_known_symbol(&self, symbol: &str) -> bool {
        self.symbol_map.contains_key(symbol)
    }

    // Returns the id for a given symbol 
    // (check if is_known_symbol, otherwise panics)
    pub fn get_symbol_id(&self, symbol: &str) -> u8 {
        let (b,_) = self.symbol_map.get(symbol).unwrap().clone();  
        b 
    }

    pub fn get_symbol_type(&self, symbol: &str) -> Type {
        let (_,b) = self.symbol_map.get(symbol).unwrap().clone();
        b
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
