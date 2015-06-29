//! The `codegen` module is for the creating of zcode from an ast

use std::collections::HashMap;
use std::error::Error;
use std::io::Write;

use backend::zcode::zfile::{FormattingState, Operand, Variable, ZOP, Zfile, Type};
use config::Config;
use frontend::ast;
use frontend::ast::ASTNode;
use frontend::evaluate_expression::evaluate_expression;
use frontend::lexer::Token::*;


pub fn generate_zcode<W: Write>(cfg: &Config, ast: ast::AST, output: &mut W) {
    let mut codegenerator = Codegen::new(cfg, ast);
    codegenerator.start_codegen();
    match output.write_all(&(*codegenerator.zfile_bytes())) {
        Err(why) => {
            panic!("Could not write to output: {}", Error::description(&why));
        },
        Ok(_) => {
            info!("Wrote zcode to output");
        }
    };
}

#[allow(dead_code)]
struct Codegen<'a> {
    cfg: &'a Config,
    ast: ast::AST,
    zfile: Zfile
}

impl<'a> Codegen<'a> {
    pub fn new(cfg: &'a Config, ast: ast::AST) -> Codegen<'a> {
        Codegen {
            cfg: cfg,
            ast: ast,
            zfile: Zfile::new_with_cfg(cfg)
        }
    }

    pub fn start_codegen(&mut self) {
        self.zfile.start();
        //self.zfile.op_quit();
        //self.zfile.routine("main", 0);

        self.ast.to_zcode(&mut self.zfile);
        
        self.zfile.op_quit();

        self.zfile.end();
    }

    pub fn zfile_bytes(&self) -> &Vec<u8> {
        &self.zfile.data.bytes
    }
}


/// add zcode based on tokens
pub fn gen_zcode<'a>(node: &'a ASTNode, mut out: &mut Zfile, mut manager: &mut CodeGenManager<'a>) -> Vec<ZOP> {
    let mut state_copy = manager.format_state.clone();
    let mut set_formatting = false;
    
    match node {
        &ASTNode::Passage(ref node) => {
            let mut code: Vec<ZOP> = vec![];
            match &node.category {
                &TokPassage {ref name, .. } => {
                    code.push(ZOP::Routine{name: name.to_string(), count_variables: 15});
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
            code.push(ZOP::Call1N{jump_to_label: "mem_free".to_string()});
            code.push(ZOP::Ret{value: Operand::new_const(0)});
            code
        },
        &ASTNode::Default(ref t) => {
            let mut code: Vec<ZOP> = match &t.category {
                &TokText {ref text, .. } => {
                    vec![ZOP::PrintOps{text: text.to_string()}]
                },
                &TokNewLine { .. } => {
                    vec![ZOP::Newline]
                },
                &TokFormatBoldStart { .. } => {
                    state_copy.bold = true;
                    set_formatting = true;
                    vec![ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic}]
                },
                &TokFormatMonoStart { .. } => {
                    state_copy.mono = true;
                    set_formatting = true;
                    vec![ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic}]
                },
                &TokFormatItalicStart { .. } => {
                    state_copy.italic = true;
                    set_formatting = true;
                    vec![ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic}]
                },
                &TokPassageLink {ref display_name, ref passage_name, .. } => {
                    set_formatting = true;
                    vec![
                    ZOP::Call2NWithAddress{jump_to_label: "system_add_link".to_string(), address: passage_name.to_string()},
                    ZOP::SetColor{foreground: 8, background: 2},
                    ZOP::Print{text: format!("{}[", display_name)},
                    ZOP::PrintNumVar{variable: Variable::new(16)},
                    ZOP::Print{text: "]".to_string()},
                    ZOP::SetColor{foreground: 9, background: 2},
                    ]
                },
                &TokAssign {ref var_name, ref op_name, .. } => {
                    if op_name == "=" || op_name == "to" {
                        let mut code: Vec<ZOP> = vec![];
                        if t.childs.len() == 1 {
                            let expression_node = &t.childs[0].as_default();
                            let result = match expression_node.category {
                                TokExpression => {
                                    if expression_node.childs.len() != 1 {
                                        panic!("Unsupported expression!")
                                    }
                                    evaluate_expression(&expression_node.childs[0], &mut code, manager, &mut out)
                                }, _ => panic!("Unsupported expression!")
                            };
                            if !manager.symbol_table.is_known_symbol(var_name) {
                                let vartype = match result {
                                    Operand::StringRef(_) => Type::String,
                                    Operand::Var(ref var) => var.vartype.clone(),
                                    _ => Type::Integer
                                };
                                manager.symbol_table.insert_new_symbol(&var_name, vartype);
                            }
                            let symbol_id = manager.symbol_table.get_symbol_id(var_name);
                            code.push(ZOP::StoreVariable{variable: symbol_id, value: result});
                            code
                        } else {
                            debug!("Assign Expression currently not supported.");
                            vec![]
                        }
                    } else { vec![] }
                },
                &TokMacroIf { .. } => {
                    if t.childs.len() < 2 {
                        panic!("Unsupported if-expression!");
                    }

                    // check if the first node is an expression node
                    let expression_node = match t.childs[0].as_default().category {
                        TokExpression => t.childs[0].as_default(),
                        _ =>  panic!("Unsupported if-expression!")
                    };

                    let mut code: Vec<ZOP> = vec![];

                    // Evaluate the contained expression
                    let result = evaluate_expression(&expression_node.childs[0], &mut code, manager, &mut out);

                    let if_id = manager.ids_if.start_next();
                    let if_label = format!("if_{}", if_id);
                    let after_if_label = format!("after_if_{}", if_id);
                    let after_else_label = format!("after_else_{}", if_id);
                    code.push(ZOP::JG{operand1: result, operand2: Operand::new_const(0), jump_to_label: if_label.to_string()});
                    code.push(ZOP::Jump{jump_to_label: after_if_label.to_string()});
                    code.push(ZOP::Label{name: if_label.to_string()});

                    for i in 1..t.childs.len() {
                        for instr in gen_zcode(&t.childs[i], out, manager) {
                            code.push(instr);
                        }
                    }

                    code.push(ZOP::Jump{jump_to_label: after_else_label});
                    code.push(ZOP::Label{name: after_if_label});
                    code
                },
                &TokMacroElseIf { .. } => {
                    if t.childs.len() < 2 {
                        panic!("Unsupported elseif-expression!");
                    }

                    let mut code: Vec<ZOP> = vec![];

                    // check if the first node is an expression node
                    let expression_node = match t.childs[0].as_default().category {
                        TokExpression => t.childs[0].as_default(),
                        _ =>  panic!("Unsupported elseif-expression!")
                    };

                    // Evaluate the contained expression
                    let result = evaluate_expression(&expression_node.childs[0], &mut code, manager, &mut out);
 
                    let if_id = manager.ids_if.start_next();

                    let if_label = format!("if_{}", if_id);
                    let after_if_label = format!("after_if_{}", manager.ids_if.pop_id());
                    let after_else_label = format!("after_else_{}", manager.ids_if.peek());
                    code.push(ZOP::JG{operand1: result, operand2: Operand::new_const(0), jump_to_label: if_label.to_string()});
                    code.push(ZOP::Jump{jump_to_label: after_if_label.to_string()});
                    code.push(ZOP::Label{name: if_label.to_string()});

                    for i in 1..t.childs.len() {
                        for instr in gen_zcode(&t.childs[i], out, manager) {
                            code.push(instr);
                        }
                    }

                    code.push(ZOP::Jump{jump_to_label: after_else_label});
                    code.push(ZOP::Label{name: after_if_label});
                    code
                },
                &TokMacroElse { .. } => {
                    let mut code: Vec<ZOP> = vec![];
                    for child in &t.childs {
                        for instr in gen_zcode(child, out, manager) {
                            code.push(instr);
                        }
                    }
                    code
                },
                &TokMacroEndIf { .. } => {
                    let after_else_label = format!("after_else_{}", manager.ids_if.pop_id());
                    vec![ZOP::Label{name: after_else_label}]
                },

                &TokMacroDisplay {ref passage_name, .. } => {
                    let var = Variable::new(17);
                    vec![
                    // activates the display-modus
                    ZOP::StoreVariable{variable: var.clone(), value: Operand::new_const(1)},
                    ZOP::Call1N{jump_to_label: passage_name.to_string()},

                    // deactivates the display-modus
                    ZOP::StoreVariable{variable: var.clone(), value: Operand::new_const(0)},
                    ]
                },
                &TokMacroPrint { .. } => {
                    if t.childs.len() != 1 {
                        panic!("Doesn't support print with 0 or more than one argument");
                    }

                    let mut code: Vec<ZOP> = vec![];

                    let child = &t.childs[0].as_default();

                    match child.category {
                        TokExpression => {
                            let eval = evaluate_expression(&child.childs[0], &mut code, manager, &mut out);
                            match eval {
                                Operand::Var(var) => if var.vartype == Type::String { code.push(ZOP::PrintUnicodeStr{address: Operand::new_var_string(var.id)}); } else { code.push(ZOP::PrintNumVar{variable: var}); },
                                Operand::StringRef(addr) => code.push(ZOP::PrintUnicodeStr{address: Operand::new_large_const(addr.value)}),
                                Operand::Const(c) => code.push(ZOP::Print{text: format!("{}", c.value)}),
                                Operand::LargeConst(c) => code.push(ZOP::Print{text: format!("{}", c.value)})
                            };
                        },
                        _ => {
                            panic!("Unsupported Expression");
                        }
                    };
                    code
                },
                &TokMacroContentVar {ref var_name, .. } => {
                    let var_id = manager.symbol_table.get_symbol_id(&*var_name);
                    match manager.symbol_table.get_symbol_type(&*var_name) {
                        Type::Integer => {
                            vec![ZOP::PrintNumVar{variable: var_id}]
                        },
                        Type::String => {
                            vec![ZOP::PrintUnicodeStr{address: Operand::new_var(var_id.id)}]
                        },
                        Type::Bool => {
                            vec![ZOP::PrintNumVar{variable: var_id}]
                        }
                    }
                },
                _ => {
                    debug!("no match if");
                    vec![]
                },
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

/// random(from, to) -> zcode op_random(0, range)
pub fn function_random(arg_from: &Operand, arg_to: &Operand,
        code: &mut Vec<ZOP>, temp_ids: &mut Vec<u8>) -> Operand {

    let range_var: Variable = match temp_ids.pop() {
        Some(var) => Variable::new(var),
        None      => panic!{"Function random has no range variable"}
    };

    // Calculate range = to - from + 1
    code.push(ZOP::Sub{
        operand1: arg_to.clone(), 
        operand2: arg_from.clone(), 
        save_variable: range_var.clone()
    });
    code.push(ZOP::Add{
        operand1: Operand::new_var(range_var.id), 
        operand2: Operand::new_const(1), 
        save_variable: range_var.clone()
    });

    let var: Variable = match temp_ids.pop() {
        Some(var) => Variable::new(var),
        None      => panic!{"Function random has no variable"}
    };

    // get a random number between 1 and range
    code.push(ZOP::Random {range: Operand::new_var(range_var.id), variable: var.clone()} );

    // add (arg_from - 1) to range (because min. random is 1 not 0)
    code.push(ZOP::Add{
        operand1: Operand::new_var(var.id), 
        operand2: arg_from.clone(), 
        save_variable: var.clone()
    });
     code.push(ZOP::Sub{
        operand1: Operand::new_var(var.id), 
        operand2: Operand::new_const(1), 
        save_variable: var.clone()
    });
    temp_ids.push(range_var.id);
    Operand::new_var(var.id)
}

pub struct CodeGenManager<'a> {
    pub ids_if: IdentifierProvider,
    pub ids_expr: IdentifierProvider,
    pub passages: Vec<String>,
    pub symbol_table: SymbolTable<'a>,
    pub format_state: FormattingState
}

pub struct IdentifierProvider {
    current_id: u32,
    id_stack: Vec<u32>
}

pub struct SymbolTable<'a> {
    current_id: u8,
    symbol_map: HashMap<&'a str, (Variable, Type)>
}

impl <'a> CodeGenManager<'a> {
    pub fn new() -> CodeGenManager<'a> {
        CodeGenManager {
            ids_if: IdentifierProvider::new(),
            ids_expr: IdentifierProvider::new(),
            passages: Vec::new(),
            symbol_table: SymbolTable::new(),
            format_state: FormattingState {bold: false, italic: false, mono: false, inverted: false},
        }
    }

    pub fn new_temp_var_vec() -> Vec<u8> {
        (2..15).collect()
    }

    pub fn is_temp_var(var: &Variable) -> bool{
        var.id > 1 && var.id < 16
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

    // Returns the last id from the stack (but retains it)
    pub fn peek(&mut self) -> u32 {
        if let Some(temp) = self.id_stack.last() {
            return temp.clone()
        }

        panic!{"id_stack is empty, peek wasn't possible."}
    }

    // Pops the last id from the stack
    pub fn pop_id(&mut self) -> u32 {
        if let Some(temp) = self.id_stack.pop() {
            return temp.clone()
        }

        panic!{"id_stack is empty, pop wasn't possible."}
    }
}

impl <'a> SymbolTable<'a> {
    pub fn new() -> SymbolTable<'a> {
        SymbolTable {
            current_id: 25,
            symbol_map: HashMap::<&str, (Variable, Type)>::new()
        }
    }

    // Inserts a symbol into the table, assigning a new id
    pub fn insert_new_symbol(&mut self, symbol: &'a str, t: Type) {
        debug!("Assigned id {} to variable {}", self.current_id, symbol);
        self.symbol_map.insert(symbol, (Variable{id: self.current_id, vartype: t.clone()}, t));
        self.current_id += 1;
    }

    // Checks if the symbol is already existent in the table
    pub fn is_known_symbol(&self, symbol: &str) -> bool {
        self.symbol_map.contains_key(symbol)
    }

    // Returns the id for a given symbol
    // (check if is_known_symbol, otherwise panics)
    pub fn get_symbol_id(&self, symbol: &str) -> Variable {
        if let Some(temp) = self.symbol_map.get(symbol) {
            return temp.0.clone()
        }

        panic!{"symbol_map is empty, get_symbol_id wasn't possible."}
    }

    pub fn get_symbol_type(&self, symbol: &str) -> Type {
        if let Some(temp) = self.symbol_map.get(symbol) {
            return temp.1.clone()
        }

        panic!{"symbol_map is empty, get get_symbol_type wasn't possible."}
    }

    pub fn has_var_id(&self, id: u8) -> bool {
        for name in self.symbol_map.keys() {
            if let Some(temp) = self.symbol_map.get(name) {
                if temp.0.clone().id == id {
                    return true;
                }
            } else {
                panic!{"symbol_map is empty, has_var_id wasn't possible."}
            }
        }
        false
    }

    pub fn get_symbol_type_by_id(&self, id: u8) -> Type {
        for name in self.symbol_map.keys() {
            if let Some(temp) = self.symbol_map.get(name) {
                if temp.0.clone().id == id {
                    return temp.1.clone();
                }
            } else {
                panic!{"symbol_map is empty, get_symbol_type_by_id wasn't possible."}
            }
        }
        panic!("should never happen: could not find the requested ID in symbol table")
    }
}
