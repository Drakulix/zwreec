//! The `ast` module contains a lot of useful functionality
//! to create and walk through the ast (abstract syntaxtree)

use frontend::expressionparser;
use frontend::lexer::Token;
use frontend::lexer::Token::*;
use backend::zcode::zfile;
use backend::zcode::zfile::{FormattingState, ZOP, Operand, Variable, Constant, LargeConstant};
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
    passages: Vec<ASTNode>,
    path: Vec<usize>,
    is_in_if_expression: bool
}

 /// add zcode based on tokens
fn gen_zcode<'a>(node: &'a ASTNode, mut out: &mut zfile::Zfile, mut manager: &mut CodeGenManager<'a>) -> Vec<ZOP> {
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
            code.push(ZOP::Call1N{jump_to_label: "system_check_links".to_string()});
            code.push(ZOP::Ret{value: 0});
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

                &Token::TokMacroDisplay {ref passage_name, .. } => {
                	let var = Variable { id: 17 };
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
                    			Operand::Var(var) => code.push(ZOP::PrintNumVar{variable: var}),
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

fn evaluate_expression<'a>(node: &'a ASTNode, code: &mut Vec<ZOP>, mut manager: &mut CodeGenManager<'a>, mut out: &mut zfile::Zfile) -> Operand {
	let mut temp_ids = CodeGenManager::new_temp_var_vec();
	evaluate_expression_internal(node, code, &mut temp_ids, manager, &mut out)
}

// Evaluates an expression node to zCode.
fn evaluate_expression_internal<'a>(node: &'a ASTNode, code: &mut Vec<ZOP>,
		temp_ids: &mut Vec<u8>, mut manager: &mut CodeGenManager<'a>, mut out: &mut zfile::Zfile) -> Operand {
	let n = node.as_default();

	match n.category {
		TokNumOp { ref op_name, .. } => {
			if n.childs.len() != 2 {
				panic!("Numeric operators need two arguments!")
			}
			let eval0 = evaluate_expression_internal(&n.childs[0], code, temp_ids, manager, &mut out);
			let eval1 = evaluate_expression_internal(&n.childs[1], code, temp_ids, manager, &mut out);
			eval_num_op(&eval0, &eval1, &**op_name, code, temp_ids)
		},
		TokCompOp { ref op_name, .. } => {
			if n.childs.len() != 2 {
				panic!("Numeric operators need two arguments!")
			}
			let eval0 = evaluate_expression_internal(&n.childs[0], code, temp_ids, manager, &mut out);
			let eval1 = evaluate_expression_internal(&n.childs[1], code, temp_ids, manager, &mut out);
			eval_comp_op(&eval0, &eval1, &**op_name, code, temp_ids, manager)
		},
		TokLogOp { ref op_name, .. } => {
			let eval0 = evaluate_expression_internal(&n.childs[0], code, temp_ids, manager, &mut out);
			
			match &**op_name {
				"and" | "or" => {
					let eval1 = evaluate_expression_internal(&n.childs[1], code, temp_ids, manager, &mut out);
					eval_and_or(&eval0, &eval1, &**op_name, code, temp_ids)
				},
				"not" => {
					eval_not(&eval0, code, temp_ids, manager)
				}, 
				_ => panic!("unhandled op")
			}
		}
		TokInt { ref value, .. } => {
			Operand::new_large_const(*value as i16)
		},
		TokBoolean { ref value, .. } => {
			boolstr_to_const(&**value)
		},
        TokString {ref value, .. } => {
            Operand::new_string_ref(out.write_string(value) as i16)
        }
		TokVariable { ref name, .. } => {
			Operand::Var(manager.symbol_table.get_symbol_id(name))
		},
		TokFunction { ref name, .. } => {
			match &**name {
				"random" => {
					let args = &node.as_default().childs;
				    if args.len() != 2 {
				        panic!("Function random only supports 2 args");
				    }

				    if args[0].as_default().childs.len() != 1 || args[1].as_default().childs.len() != 1 {
				        panic!("Unsupported Expression");
				    }

				    let from = &args[0].as_default().childs[0];
				    let to = &args[1].as_default().childs[0];

				    let from_value = evaluate_expression_internal(from, code, temp_ids, manager, &mut out);
				    let to_value = evaluate_expression_internal(to, code, temp_ids, manager, &mut out);
					function_random(&from_value, &to_value, code, temp_ids)
				},
				_ => { panic!("Unsupported function: {}", name)}
			}
		},
		_ => panic!("unhandled token {:?}", n.category)
	}
}

fn eval_num_op<'a>(eval0: &Operand, eval1: &Operand, op_name: &str, code: &mut Vec<ZOP>, temp_ids: &mut Vec<u8>) -> Operand {
	if count_constants(eval0, eval1) == 2 {
		return direct_eval_num_op(eval0, eval1, op_name);
	}
	let save_var = determine_save_var(eval0, eval1, temp_ids);
	match op_name {
		"+" => {
			code.push(ZOP::Add{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
		},
		"-" => {
			code.push(ZOP::Sub{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
		},
		"*" => {
            code.push(ZOP::Mul{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
		},
		"/" => {
            code.push(ZOP::Div{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
		},
		"%" => {
            code.push(ZOP::Mod{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
		},
		_ => panic!("unhandled op")
	};

	free_var_if_both_temp(eval0, eval1, temp_ids);

	Operand::Var(save_var)
}



fn direct_eval_num_op(eval0: &Operand, eval1: &Operand, op_name: &str) -> Operand {
	let mut out_large = false;
	let val0 = eval0.const_value();
	let val1 = eval1.const_value();
	match eval0 { &Operand::LargeConst(_) => {out_large = true; }, _ => {} };
	match eval1 { &Operand::LargeConst(_) => {out_large = true; }, _ => {} };
	let result = match op_name {
		"+" => {
			val0 + val1
		},
		"-" => {
			val0 - val1
		},
		"*" => {
			val0 * val1
		},
		"/" => {
			val0 / val1
		},
		"%" => {
			val0 % val1
		},
		_ => panic!("unhandled op")
	};
	if out_large {
		Operand::LargeConst(LargeConstant { value: result })
	} else {
		Operand::Const(Constant { value: result as u8 })
	}
}

fn eval_comp_op<'a>(eval0: &Operand, eval1: &Operand, op_name: &str, code: &mut Vec<ZOP>, 
		temp_ids: &mut Vec<u8>, mut manager: &mut CodeGenManager<'a>) -> Operand {
	if count_constants(eval0, eval1) == 2 {
		return direct_eval_comp_op(eval0, eval1, op_name);
	}
	let save_var = Variable { id: temp_ids.pop().unwrap() };
	let label = format!("expr_{}", manager.ids_expr.start_next());
	let const_true = Operand::new_const(1);
	let const_false = Operand::new_const(0);
	match op_name {
		"is" | "==" | "eq" => {
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true});
			code.push(ZOP::JE{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false});
		},
		"neq" => {
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false});
			code.push(ZOP::JE{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true});
		},
		"<" | "lt" =>  {
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true });
			code.push(ZOP::JL{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false});
		},
		"<=" | "lte" => {
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false});
			code.push(ZOP::JG{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true});
		},
		">=" | "gte" => {
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false});
			code.push(ZOP::JL{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true});
		},
		">" | "gt" => {
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true});
			code.push(ZOP::JG{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
			code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false});
		},
		_ => panic!("unhandled op")
	};
	code.push(ZOP::Label {name: label.to_string()});
	free_var_if_temp(eval0, temp_ids);
	free_var_if_temp(eval1, temp_ids);
	Operand::Var(save_var)
}

/// Directly evaluates the given compare operation.
/// Both operands must be constants.
fn direct_eval_comp_op(eval0: &Operand, eval1: &Operand, op_name: &str) -> Operand {
	let val0 = eval0.const_value();
	let val1 = eval1.const_value();
	let result = match op_name {
		"is" | "==" | "eq" => { val0 == val1 },
		"neq" => { val0 != val1	},
		"<" | "lt" =>  { val0 < val1 },
		"<=" | "lte" => { val0 <= val1 },
		">=" | "gte" => { val0 >= val1 },
		">" | "gt" => { val0 > val1 },
		_ => panic!("unhandled op")
	};
	if result {
		Operand::Const(Constant {value: 1})
	} else {
		Operand::Const(Constant {value: 0})
	}
}

fn eval_and_or(eval0: &Operand, eval1: &Operand, op_name: &str, code: &mut Vec<ZOP>, 
		temp_ids: &mut Vec<u8>) -> Operand {
	if count_constants(&eval0, &eval1) == 2 {
		let mut out_large = false;
		let val0 = eval0.const_value();
		let val1 = eval1.const_value();
		match eval0 { &Operand::LargeConst(_) => {out_large = true; }, _ => {} };
		match eval1 { &Operand::LargeConst(_) => {out_large = true; }, _ => {} };
		let result = if op_name == "or" {
				val0 | val1
			} else {
				val0 & val1
			};
		if out_large {
			return Operand::LargeConst(LargeConstant { value: result })
		} else {
			return Operand::Const(Constant { value: result as u8 })
		}
	}
	
	let save_var = determine_save_var(eval0, eval1, temp_ids);
	if op_name == "or" {
		code.push(ZOP::Or{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
	} else {
		code.push(ZOP::And{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
	}
	free_var_if_both_temp(eval0, eval1, temp_ids);
	Operand::Var(save_var)
}

fn eval_not<'a>(eval: &Operand, code: &mut Vec<ZOP>,
		temp_ids: &mut Vec<u8>, mut manager: &mut CodeGenManager<'a>) -> Operand {
	if eval.is_const() {
		let val = eval.const_value();
		let result: u8 = if val > 0 { 0 } else { 1 };
		return Operand::Const(Constant { value: result });
	}
	let save_var = Variable { id: temp_ids.pop().unwrap() };
	let label = format!("expr_{}", manager.ids_expr.start_next());
	code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: Operand::new_const(0)});
	code.push(ZOP::JG{operand1: eval.clone(), operand2: Operand::new_const(0), jump_to_label: label.to_string()});
	code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: Operand::new_const(1)});
	code.push(ZOP::Label {name: label.to_string()});
	free_var_if_temp(eval, temp_ids);
	Operand::Var(save_var)
}

fn function_random(arg_from: &Operand, arg_to: &Operand,
		code: &mut Vec<ZOP>, temp_ids: &mut Vec<u8>) -> Operand {

	let range_var = Variable::new(temp_ids.pop().unwrap());
	// Calculate range = to - from + 1
	code.push(ZOP::Sub{
		operand1: arg_to.clone(), 
		operand2: arg_from.clone(), 
		save_variable: range_var.clone()
	});
	code.push(ZOP::Sub{
		operand1: Operand::new_var(range_var.id), 
		operand2: Operand::new_const(1), 
		save_variable: range_var.clone()
	});

    let var = Variable::new(temp_ids.pop().unwrap());

    // get a random number between 0 and range
    code.push(ZOP::Random {range: Operand::new_var(range_var.id), variable: var.clone()} );

    // add arg_from to range
    code.push(ZOP::Add{
    	operand1: Operand::new_var(var.id), 
    	operand2: arg_from.clone(), 
    	save_variable: var.clone()
    });
    temp_ids.push(range_var.id);
    Operand::new_var(var.id)
}

fn free_var_if_both_temp (eval0: &Operand, eval1: &Operand, temp_ids: &mut Vec<u8>) {
	match eval0 { 
		&Operand::Var(ref var1) => {
			if CodeGenManager::is_temp_var(var1) {
				match eval1 {
					&Operand::Var(ref var2)=> {
						if CodeGenManager::is_temp_var(var2) {
							temp_ids.push(var2.id);
						}
					}, _ => {}
				}
			}
		}, _ => {}
	};
}

fn free_var_if_temp (operand: &Operand, temp_ids: &mut Vec<u8>) {
	match operand {
		&Operand::Var(ref var) => {
			if CodeGenManager::is_temp_var(var){
				temp_ids.push(var.id);
			}
		}, _ => {}
	}
}

fn determine_save_var(operand1: &Operand, operand2: &Operand, temp_ids: &mut Vec<u8>) -> Variable {
	match operand1 {
		&Operand::Var(ref var) => {
			if CodeGenManager::is_temp_var(var) {
				return var.clone();
			}
		}, _ => {}
	};
	match operand2 {
		&Operand::Var(ref var) => {
			if CodeGenManager::is_temp_var(var) {
				return var.clone();
			}
		}, _ => {}
	};
	return Variable { id: temp_ids.pop().unwrap() };
}

fn count_constants(operand1: &Operand, operand2: &Operand) -> u8 {
	let mut const_count: u8 = 0;
	if operand1.is_const() {
		const_count += 1;
	}
	if operand2.is_const() {
		const_count += 1;
	}
	const_count
}

fn boolstr_to_const(string: &str) -> Operand {
    match string {
        "true" => Operand::Const(Constant { value: 1 }),
        _ => Operand::Const(Constant { value: 0 })
    }
}

pub enum ASTOperation {
    AddPassage(Token),
    AddChild(Token),
    ChildDown(Token),
    Up,
    UpChild(Token),
    UpChildDown(Token),
    UpSpecial,
}

impl AST {
    pub fn build<I: Iterator<Item=ASTOperation>>(ops: I) -> AST {
        let mut ast = AST {
            passages: Vec::new(),
            path: Vec::new(),
            is_in_if_expression: false,
        };
        for op in ops {
            ast.operation(op);
        }
        ast.parse_expressions();

        ast
    }

    pub fn operation(&mut self, op: ASTOperation) {
        use self::ASTOperation::*;
        match op {
            AddPassage(passage) => self.add_passage(passage),
            AddChild(child) => self.add_child(child),
            ChildDown(child) => self.child_down(child),
            Up => self.up(),
            UpChild(child) => self.up_child(child),
            UpChildDown(child) => self.up_child_down(child),
            UpSpecial => self.up_special(),
        }
    }

    /// goes through the whole tree and parse the expressions
    fn parse_expressions(&mut self) {
        for child in &mut self.passages {
            child.parse_expressions();
        }
    }

    /// adds a passage to the path in the ast
    pub fn add_passage(&mut self, token: Token) {
        self.path.clear();
        let ast_count_passages = self.count_childs(self.path.to_vec());

        let node = ASTNode::Passage(NodePassage { category: token, childs: Vec::new() });
        self.passages.push(node);

        self.path.push(ast_count_passages);
    }

    /// adds a child to the path in the ast
    pub fn add_child(&mut self, token: Token) {
        if let Some(index) = self.path.first() {
            let mut new_path: Vec<usize> = self.path.to_vec();
            new_path.remove(0);
            self.passages[*index].add_child(new_path, token);
        } else {
            self.passages.push(ASTNode::Default(NodeDefault { category: token, childs: Vec::new() }));
        }
    }

    /// adds a child an goees one child down
    pub fn child_down(&mut self, token: Token) {
        // 
        if token.clone() == (TokMacroIf { location: (0, 0) }) ||
           token.clone() == (TokMacroElseIf { location: (0, 0) }) {
            self.is_in_if_expression = true;
        }

        let ast_count_childs = self.count_childs(self.path.to_vec());
        self.add_child(token);
        self.path.push(ast_count_childs);
    }

    /// adds one child and goes down. adds snd child and goes down.
    pub fn two_childs_down(&mut self, child1: Token, child2: Token) {
        self.child_down(child1);
        self.child_down(child2);
    }

    /// goes one lvl up
    pub fn up(&mut self) {
        self.path.pop();
    }

    /// special up of the if-expression
    pub fn up_special(&mut self) {
        if !self.is_in_if_expression {
            self.path.pop();
        } else {
            self.is_in_if_expression = false;
        }
    }

    /// goes one lvl up and adds and child
    pub fn up_child(&mut self, token: Token) {
        self.up();
        self.add_child(token);
    }

    /// goes one lvl up, adds an child and goes one lvl down
    pub fn up_child_down(&mut self, token: Token) {
        self.up();
        self.child_down(token);
    }

    /// goes two lvl up
    pub fn two_up(&mut self) {
        self.up();
        self.up();
    }


    /// convert ast to zcode
    pub fn to_zcode(&self, out: &mut zfile::Zfile) {
        let mut manager = CodeGenManager::new();

        // Insert temp variables for internal calculations
        manager.symbol_table.insert_new_symbol("int0", Type::Integer);

        let mut code: Vec<ZOP> = vec![];
        for child in &self.passages {
            for instr in gen_zcode(child, out, &mut manager) {
                code.push(instr);
            }
        }
        out.emit(code);
    }

    /// prints the tree
    pub fn print(&self, force_print: bool) {
        debug!("Abstract Syntax Tree: ");
        for child in &self.passages {
            child.print(0, force_print);
        }
        debug!("");
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

    /// checks in the ast if there is the token "token"
    pub fn is_specific_token(&self, token: Token, path: Vec<usize>) -> bool {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            self.passages[*index].is_specific_token(token, new_path)
        } else {
            false
        }
    }
}

// ================================
// node types
#[derive(Clone)]
pub enum ASTNode {
    Default (NodeDefault),
    Passage (NodePassage)
}

#[derive(Clone)]
pub struct NodePassage {
    category: Token,
    pub childs: Vec<ASTNode>,
    /*tags: Vec<ASTNode>*/
}

#[derive(Clone)]
pub struct NodeDefault {
    pub category: Token,
    pub childs: Vec<ASTNode>
}

struct CodeGenManager<'a> {
    ids_if: IdentifierProvider,
    ids_expr: IdentifierProvider,
    symbol_table: SymbolTable<'a>,
    format_state: FormattingState,
}

struct IdentifierProvider {
    current_id: u32,
    id_stack: Vec<u32>
}

struct SymbolTable<'a> {
    current_id: u8,
    symbol_map: HashMap<&'a str, (Variable, Type)>
}

impl <'a> CodeGenManager<'a> {
    pub fn new() -> CodeGenManager<'a> {
        CodeGenManager {
            ids_if: IdentifierProvider::new(),
            ids_expr: IdentifierProvider::new(),
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

    // Pops the last id from the stack
    pub fn peek(&mut self) -> u32 {
        self.id_stack.last().unwrap().clone()
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
            symbol_map: HashMap::<&str, (Variable, Type)>::new()
        }
    }

    // Inserts a symbol into the table, assigning a new id
    pub fn insert_new_symbol(&mut self, symbol: &'a str, t: Type) {
        debug!("Assigned id {} to variable {}", self.current_id, symbol);
        self.symbol_map.insert(symbol, (Variable::new(self.current_id),t));
        self.current_id += 1;
    }

    // Checks if the symbol is already existent in the table
    pub fn is_known_symbol(&self, symbol: &str) -> bool {
        self.symbol_map.contains_key(symbol)
    }

    // Returns the id for a given symbol
    // (check if is_known_symbol, otherwise panics)
    pub fn get_symbol_id(&self, symbol: &str) -> Variable {
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

    /// checks the current path if there is the token "token"
    pub fn is_specific_token(&self, token: Token, path: Vec<usize>) -> bool {
        if let Some(index) = path.first() {
            let mut new_path: Vec<usize> = path.to_vec();
            new_path.remove(0);

            match self {
                &ASTNode::Default(ref node) => node.childs[*index].is_specific_token(token, new_path),
                &ASTNode::Passage(ref node) => node.childs[*index].is_specific_token(token, new_path),
            }
        } else {
            match self {
                &ASTNode::Default(ref node) => {
                    token == node.category
                },
                &ASTNode::Passage(ref node) => {
                    token == node.category
                },
            }
        }
    }

    pub fn category(&self) -> Token {
        match self {
            &ASTNode::Passage(ref t) => {
                t.category.clone()
            },
            &ASTNode::Default(ref t) => {
                t.category.clone()
            }
        }
    }

    /// prints an node of an ast
    pub fn print(&self, indent: usize, force_print: bool) {
        let mut spaces = "".to_string();
        for _ in 0..indent {
            spaces.push_str(" ");
        }

        match self {
            &ASTNode::Passage(ref t) => {
                if force_print {
                    println!("{}|- : {:?}", spaces, t.category);
                } else {
                    debug!("{}|- : {:?}", spaces, t.category);
                }
                for child in &t.childs {
                    child.print(indent+2, force_print);
                }
            },
            &ASTNode::Default(ref t) => {
                if force_print {
                    println!("{}|- : {:?}", spaces, t.category);
                } else {
                    debug!("{}|- : {:?}", spaces, t.category);
                }
                for child in &t.childs {
                    child.print(indent+2, force_print);
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

    /// goes through the whole tree and parse the expressions
    fn parse_expressions(&mut self) {
        match self {
            &mut ASTNode::Passage(ref mut node) => {
                for mut child in node.childs.iter_mut() {
                    child.parse_expressions();
                }
            },
            &mut ASTNode::Default(ref mut node) => {
                match &node.category {
                    &TokExpression => {
                        expressionparser::ExpressionParser::parse(node);
                    },
                    _ => ()
                }

                for mut child in node.childs.iter_mut() {
                    child.parse_expressions();
                }
            }
        }
    }    
}

// ================================
// test functions
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use frontend::*;
    use frontend::lexer::Token;
    use frontend::lexer::Token::*;
    use config::Config;

    /// creates an ast from the inputs str
    fn test_ast(input: &str) -> AST {
        let cfg = Config::default_config();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(input.to_string().into_bytes());
        let tokens = lexer::lex(&cfg, &mut cursor);
        let parser = parser::Parser::new(&cfg);
        AST::build(parser.parse(tokens.inspect(|ref token| {
            println!("{:?}", token);
        })))
    }

    /// checks exptexted
    fn test_expected(expected: Vec<(Vec<usize>, Token)>, ast: AST) {
        for item in expected.iter() {
            let b = ast.is_specific_token(item.1.clone(), item.0.to_vec());
            if b == false {
                ast.print(true);
            }
            assert!(ast.is_specific_token(item.1.clone(), item.0.to_vec()));
        }
    }

    #[test]
    fn text_test() {
        let ast = test_ast("::Passage\nTestText\nTestNextLine\n::NextPassage");

        let expected = vec!(
            (vec![0]  , TokPassage {name: "Passage".to_string(), location: (0, 0)}),
            (vec![0,0], TokText {location: (0, 0), text: "TestText".to_string()}),
            (vec![0,1], TokNewLine {location: (0, 0)} ),
            (vec![0,2], TokText {location: (0, 0), text: "".to_string()}),
            (vec![0,1], TokNewLine {location: (0, 0)}),
            (vec![1]  , TokPassage {name: "".to_string(), location: (0, 0)}),

        );

        test_expected(expected, ast);
    }

    #[test]
    fn test_expression() {
        let ast = test_ast("::Passage\n<<print 1-2*3-4*5>>");

        let expected = vec!(
            (vec![0]            , TokPassage { location: (0, 0), name: "Passage".to_string() }),
            (vec![0,0]          , TokMacroPrint { location: (0, 0) }),
            (vec![0,0,0]        , TokExpression),
            (vec![0,0,0,0]      , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,0,0,0,0]    , TokNumOp { location: (0, 0), op_name: "-".to_string() }),
            (vec![0,0,0,0,0,0]  , TokInt { location: (0, 0), value: 1 }),
            (vec![0,0,0,0,0,1]  , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,0,0,0,0,1,0], TokInt { location: (0, 0), value: 2 }),
            (vec![0,0,0,0,0,1,1], TokInt { location: (0, 0), value: 3}),
            (vec![0,0,0,0,1]    , TokNumOp { location: (0, 0), op_name: "*".to_string() }),
            (vec![0,0,0,0,1,0]  , TokInt { location: (0, 0), value: 4 }),
            (vec![0,0,0,0,1,1]  , TokInt { location: (0, 0), value: 5 }),
        );

        test_expected(expected, ast);
    }
}
