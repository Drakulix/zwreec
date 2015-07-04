//! The `codegen` module is for the creating of zcode from an ast

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::Write;

use backend::zcode::zfile::{Constant, FormattingState, Operand, Variable, ZOP, Zfile, Type};
use config::Config;
use frontend::ast;
use frontend::ast::ASTNode;
use frontend::evaluate_expression::{evaluate_expression, EvaluateExpressionError};
use frontend::lexer::Token;
use frontend::lexer::Token::*;

#[derive(Debug)]
pub enum CodeGenError {
    CouldNotWriteToOutput { why: String },
    InvalidAST,
    NoMatch { token: Token },
    NoStartPassage,
    PassageDoesNotExist { name: String, token: Token },
    UnsupportedExpression { token: Token },
    UnsupportedIfExpression { token: Token },
    UnsupportedElseIfExpression { token: Token },
    UnsupportedExpressionType { name: String },
    UnsupportedLongExpression { name: String, token: Token },
    IdentifierStackEmpty,
    SymbolMapEmpty,
    CouldNotFindSymbolId,
}

pub fn generate_zcode<W: Write>(cfg: &Config, ast: ast::AST, output: &mut W) {
    let mut codegenerator = Codegen::new(cfg, ast);
    codegenerator.start_codegen();
    match output.write_all(&(*codegenerator.zfile_bytes())) {
        Err(why) => {
            error_panic!(cfg => CodeGenError::CouldNotWriteToOutput { why: Error::description(&why).to_string() } );
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

        self.ast_to_zcode();
        
        self.zfile.op_quit();

        self.zfile.end();
    }

    pub fn zfile_bytes(&self) -> &Vec<u8> {
        &self.zfile.data.bytes
    }

    /// convert ast to zcode
    pub fn ast_to_zcode(&mut self) {
        let mut manager = CodeGenManager::new(self.cfg);

        // adds a vec of passagenames to the manager
        manager.passages = self.ast.passage_nodes_to_string();

        // Insert temp variables for internal calculations
        manager.symbol_table.insert_new_symbol("int0", Type::Integer);

        let mut code: Vec<ZOP> = vec![];
        for child in &self.ast.passages {
            for instr in gen_zcode(child, &mut self.zfile, &mut manager) {
                code.push(instr);
            }
        }

        self.zfile.emit(code);
    }
}


/// add zcode based on tokens
pub fn gen_zcode<'a>(node: &'a ASTNode, mut out: &mut Zfile, mut manager: &mut CodeGenManager<'a>) -> Vec<ZOP> {
    let mut state_copy = manager.format_state.clone();
    let mut set_formatting = false;
    let cfg = manager.cfg;

    // Check for start passage. If it does not exist abort.
    if !manager.passages.contains(&"Start".to_string()) {
        error_force_panic!(CodeGenError::NoStartPassage);
    }
    
    match node {
        &ASTNode::Passage(ref node) => {
            let mut code: Vec<ZOP> = vec![];
            match &node.category {
                &TokPassage {ref name, .. } => {
                    code.push(ZOP::Routine{name: name.to_string(), count_variables: 15});
                },
                _ => {
                    error_panic!(cfg => CodeGenError::InvalidAST);
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

                    if !manager.is_silent {
                        vec![ZOP::PrintOps{text: text.to_string()}]
                    } else {
                        vec![]
                    }
                },
                &TokNewLine { .. } => {
                    if !manager.is_silent && !manager.is_nobr {
                        vec![ZOP::Newline]
                    } else {
                        vec![]
                    }
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
                &TokMacroSilently { .. } => {
                    manager.is_silent = true;
                    let mut code: Vec<ZOP> = vec![];
                    for child in &t.childs {
                        for instr in gen_zcode(child, out, manager) {
                            code.push(instr);
                        }
                    }
                    code
                },
                &TokMacroEndSilently { .. } => {
                    manager.is_silent = false;
                    vec![]
                },
                &TokMacroNoBr { .. } => {
                    manager.is_nobr = true;
                    let mut code: Vec<ZOP> = vec![];
                    for child in &t.childs {
                        for instr in gen_zcode(child, out, manager) {
                            code.push(instr);
                        }
                    }
                    code
                },
                &TokMacroEndNoBr { .. } => {
                    manager.is_nobr = false;
                    vec![]
                },
                &TokPassageLink {ref display_name, ref passage_name, .. } => {
                    if !manager.is_silent {
                        set_formatting = true;

                        if manager.passages.contains(passage_name) {
                            vec![
                            ZOP::Call2NWithAddress{jump_to_label: "system_add_link".to_string(), address: passage_name.to_string()},
                            ZOP::SetColor{foreground: 8, background: 2},
                            ZOP::PrintOps{text: format!("{}[", display_name)},
                            ZOP::PrintNumVar{variable: Variable::new(16)},
                            ZOP::Print{text: "]".to_string()},
                            ZOP::SetColor{foreground: 9, background: 2},
                            ]
                        } else {
                            error_panic!(cfg => CodeGenError::PassageDoesNotExist { name: passage_name.clone(), token: t.category.clone() });
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                },
                &TokAssign {ref var_name, ref op_name, .. } => {
                    let mut code: Vec<ZOP> = vec![];
                    if t.childs.len() != 1 {
                        return vec![];
                    }
                    let expression_node = &t.childs[0].as_default();
                    let result = match expression_node.category {
                        TokExpression => {
                            if expression_node.childs.len() != 1 {
                                error_panic!(cfg => CodeGenError::UnsupportedExpression { token: expression_node.category.clone() } );
                            }
                            evaluate_expression(&expression_node.childs[0], &mut code, manager, &mut out)
                        }, _ => error_force_panic!(CodeGenError::UnsupportedExpression { token: expression_node.category.clone() } )
                    };
                    if !manager.symbol_table.is_known_symbol(var_name) {
                        let vartype = match result {
                            Operand::StringRef(_) => Type::String,
                            Operand::Var(ref var) => var.vartype.clone(),
                            Operand::BoolConst(_) => Type::Bool,
                            _ => Type::Integer
                        };
                        manager.symbol_table.insert_new_symbol(&var_name, vartype);
                    }
                    let symbol_id = manager.symbol_table.get_symbol_id(var_name);
                    match &**op_name {
                        "=" | "to" => { code.push(ZOP::StoreVariable{variable: symbol_id.clone(), value: result.clone()});
                                        code.push(ZOP::CopyVarType{variable: symbol_id.clone(), from: result});
                                      },
                        "+=" => {   // using temp local variables which are not the result's variable
                                    let tmp1: u8 = match result {
                                        Operand::Var(ref var) => if var.id < 3 { 15 } else { 2 },
                                        _ => 15
                                    };
                                    let tmp2: u8 = tmp1-1;
                                    code.push(ZOP::AddTypes{operand1: Operand::new_var(symbol_id.id), operand2: result, tmp1: Variable::new(tmp1), tmp2: Variable::new(tmp2), save_variable: symbol_id.clone()});
                                    },
                        "-=" => { code.push(ZOP::Sub{operand1: Operand::new_var(symbol_id.id), operand2: result, save_variable: symbol_id.clone()});
                                  code.push(ZOP::SetVarType{variable: Variable::new(symbol_id.id), vartype: Type::Integer}); },
                        "*=" => { code.push(ZOP::Mul{operand1: Operand::new_var(symbol_id.id), operand2: result, save_variable: symbol_id.clone()});
                                  code.push(ZOP::SetVarType{variable: Variable::new(symbol_id.id), vartype: Type::Integer}); },
                        "/=" =>  {code.push(ZOP::Div{operand1: Operand::new_var(symbol_id.id), operand2: result, save_variable: symbol_id.clone()});
                                  code.push(ZOP::SetVarType{variable: Variable::new(symbol_id.id), vartype: Type::Integer}); },
                        _ => {}
                    };
                    
                    code
                },
                &TokMacroIf { .. } => {
                    if t.childs.len() < 2 {
                        error_panic!(cfg => CodeGenError::UnsupportedIfExpression { token: t.category.clone() } );
                    }

                    // check if the first node is an expression node
                    let expression_node = match t.childs[0].as_default().category {
                        TokExpression => t.childs[0].as_default(),
                        _ =>  {
                            error_force_panic!(CodeGenError::UnsupportedIfExpression { token: t.category.clone() } );
                        }
                    };

                    let mut code: Vec<ZOP> = vec![];

                    // Evaluate the contained expression
                    let result = evaluate_expression(&expression_node.childs[0], &mut code, manager, &mut out);

                    let if_id = manager.ids_if.start_next();
                    let if_label = format!("if_{}", if_id);
                    let after_if_label = format!("after_if_{}", if_id);
                    let after_else_label = format!("after_else_{}", if_id);
                    code.push(ZOP::JNE{operand1: result, operand2: Operand::new_const(0), jump_to_label: if_label.to_string()});
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
                        error_panic!(cfg => CodeGenError::UnsupportedElseIfExpression { token: t.category.clone() } );
                    }

                    let mut code: Vec<ZOP> = vec![];

                    // check if the first node is an expression node
                    let expression_node = match t.childs[0].as_default().category {
                        TokExpression => t.childs[0].as_default(),
                        _ => {
                            error_force_panic!(CodeGenError::UnsupportedElseIfExpression { token: t.category.clone() } );
                        }
                    };

                    // Evaluate the contained expression
                    let result = evaluate_expression(&expression_node.childs[0], &mut code, manager, &mut out);
 
                    let if_id = manager.ids_if.start_next();

                    let if_label = format!("if_{}", if_id);
                    let after_if_label = format!("after_if_{}", manager.ids_if.pop_id());
                    let after_else_label = format!("after_else_{}", manager.ids_if.peek());
                    code.push(ZOP::JNE{operand1: result, operand2: Operand::new_const(0), jump_to_label: if_label.to_string()});
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

                    // Check if the passage exists
                    if manager.passages.contains(passage_name) {
                        vec![
                        // activates the display-mode
                        ZOP::StoreVariable{variable: var.clone(), value: Operand::new_const(1)},
                        ZOP::Call1N{jump_to_label: passage_name.to_string()},

                        // deactivates the display-mode
                        ZOP::StoreVariable{variable: var.clone(), value: Operand::new_const(0)},
                        ]
                    } else {
                        error_panic!(cfg => CodeGenError::PassageDoesNotExist { name: passage_name.clone(), token: t.category.clone() });
                        vec![]
                    }

                },
                &TokMacroPrint { .. } => {
                    if t.childs.len() != 1 {
                        error_force_panic!(CodeGenError::UnsupportedLongExpression { name: "print".to_string(), token: t.category.clone() });
                    }

                    let mut code: Vec<ZOP> = vec![];

                    if !manager.is_silent {
                        let child = &t.childs[0].as_default();

                        match child.category {
                            TokExpression => {
                                let eval = evaluate_expression(&child.childs[0], &mut code, manager, &mut out);
                                match eval {
                                    Operand::Var(var) => code.push(ZOP::PrintVar{variable: var}),
                                    Operand::StringRef(addr) => code.push(ZOP::PrintUnicodeStr{address: Operand::new_large_const(addr.value)}),
                                    Operand::Const(c) => code.push(ZOP::Print{text: format!("{}", c.value)}),
                                    Operand::LargeConst(c) => code.push(ZOP::Print{text: format!("{}", c.value)}),
                                    Operand::BoolConst(c) => if c.value == 0 { code.push(ZOP::Print{text: "false".to_string()}); } else { code.push(ZOP::Print{text: "true".to_string()}); } ,
                                };
                            },
                            _ => {
                                error_panic!(cfg => CodeGenError::UnsupportedExpression { token: child.category.clone() } );
                            }
                        };
                    }
                    code
                },
                &TokMacroContentVar {ref var_name, .. } => {
                    let var_id = manager.symbol_table.get_and_add_symbol_id(&*var_name);
                    vec![ZOP::PrintVar{variable: var_id}]
                },
                _ => {
                    error_panic!(cfg => CodeGenError::NoMatch { token: t.category.clone() } );
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
pub fn function_random(manager: &CodeGenManager, arg_from: &Operand, arg_to: &Operand,
        code: &mut Vec<ZOP>, temp_ids: &mut Vec<u8>, location: (u64, u64)) -> Operand {

    let range_var: Variable = match temp_ids.pop() {
        Some(var) => Variable::new(var),
        None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
    };

    match arg_from {
        &Operand::Var(ref var) => {
            if var.vartype != Type::Integer {
                error_panic!(manager.cfg =>EvaluateExpressionError::UnsupportedFunctionArgType { name: "random".to_string(),
                    index: 0, location: location } );
                return Operand::Const(Constant { value: 0 })
            }
        }
        &Operand::StringRef(_) => {
            error_panic!(manager.cfg =>EvaluateExpressionError::UnsupportedFunctionArgType { name: "random".to_string(),
                index: 0, location: location } );
            return Operand::Const(Constant { value: 0 })
        }
        _ => {
            // type from is fine
        }
    }

    match arg_to {
        &Operand::Var(ref var) => {
            if var.vartype != Type::Integer {
                error_panic!(manager.cfg =>EvaluateExpressionError::UnsupportedFunctionArgType { name: "random".to_string(),
                    index: 1, location: location } );
                return Operand::Const(Constant { value: 0 })
            }
        }
        &Operand::StringRef(_) => {
            error_panic!(manager.cfg =>EvaluateExpressionError::UnsupportedFunctionArgType { name: "random".to_string(),
                index: 0, location: location } );
            return Operand::Const(Constant { value: 0 })
        }
        _ => {
            // type to is fine
        }
    }

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
        None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
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
    code.push(ZOP::SetVarType{variable: var.clone(), vartype: Type::Integer});
    temp_ids.push(range_var.id);
    Operand::new_var(var.id)
}

pub struct CodeGenManager<'a> {
    pub cfg: &'a Config,
    pub ids_if: IdentifierProvider,
    pub ids_expr: IdentifierProvider,
    pub passages: HashSet<String>,
    pub symbol_table: SymbolTable<'a>,
    pub format_state: FormattingState,
    pub is_silent: bool,
    pub is_nobr: bool
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
    pub fn new(cfg: &'a Config) -> CodeGenManager<'a> {
        CodeGenManager {
            cfg: cfg,
            ids_if: IdentifierProvider::new(),
            ids_expr: IdentifierProvider::new(),
            passages: HashSet::new(),
            symbol_table: SymbolTable::new(),
            format_state: FormattingState {bold: false, italic: false, mono: false, inverted: false},
            is_silent: false,
            is_nobr: false
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

        error_force_panic!(CodeGenError::IdentifierStackEmpty);
    }

    // Pops the last id from the stack
    pub fn pop_id(&mut self) -> u32 {
        if let Some(temp) = self.id_stack.pop() {
            return temp.clone()
        }

        error_force_panic!(CodeGenError::IdentifierStackEmpty);
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

        error_force_panic!(CodeGenError::SymbolMapEmpty)
    }

    // Returns the id for a given symbol
    // (check if is_known_symbol, otherwise adds it silently as type None)
    pub fn get_and_add_symbol_id(&mut self, symbol: &'a str) -> Variable {
        if !self.symbol_map.contains_key(symbol) {
            self.insert_new_symbol(symbol, Type::None);
        }
        if let Some(temp) = self.symbol_map.get(symbol) {
            return temp.0.clone()
        }
        error_force_panic!(CodeGenError::SymbolMapEmpty)
    }

    pub fn get_symbol_type(&self, symbol: &str) -> Type {
        if let Some(temp) = self.symbol_map.get(symbol) {
            return temp.1.clone()
        }

        error_force_panic!(CodeGenError::SymbolMapEmpty)
    }

    pub fn has_var_id(&self, id: u8) -> bool {
        for name in self.symbol_map.keys() {
            if let Some(temp) = self.symbol_map.get(name) {
                if temp.0.clone().id == id {
                    return true;
                }
            } else {
                error_force_panic!(CodeGenError::SymbolMapEmpty)
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
                error_force_panic!(CodeGenError::SymbolMapEmpty)
            }
        }

        error_force_panic!(CodeGenError::CouldNotFindSymbolId)
    }
}
