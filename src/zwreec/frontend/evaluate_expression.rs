//! The `evaluate_expressions` module...


use backend::zcode::zfile::{ZOP, Operand, Variable, Constant, LargeConstant};
use frontend::ast::{ASTNode};
use frontend::codegen;
use frontend::codegen::{CodeGenManager};
use frontend::lexer::Token::{TokNumOp, TokCompOp, TokLogOp, TokInt, TokBoolean, TokVariable, TokFunction};



pub fn evaluate_expression<'a>(node: &'a ASTNode, code: &mut Vec<ZOP>, mut manager: &mut CodeGenManager<'a>) -> Operand {
    let mut temp_ids = CodeGenManager::new_temp_var_vec();
    evaluate_expression_internal(node, code, &mut temp_ids, manager)
}

/// Evaluates an expression node to zCode.
fn evaluate_expression_internal<'a>(node: &'a ASTNode, code: &mut Vec<ZOP>,
        temp_ids: &mut Vec<u8>, mut manager: &mut CodeGenManager<'a>) -> Operand {
    let n = node.as_default();

    match n.category {
        TokNumOp { ref op_name, .. } => {
            if n.childs.len() != 2 {
                panic!("Numeric operators need two arguments!")
            }
            let eval0 = evaluate_expression_internal(&n.childs[0], code, temp_ids, manager);
            let eval1 = evaluate_expression_internal(&n.childs[1], code, temp_ids, manager);
            eval_num_op(&eval0, &eval1, &**op_name, code, temp_ids)
        },
        TokCompOp { ref op_name, .. } => {
            if n.childs.len() != 2 {
                panic!("Numeric operators need two arguments!")
            }
            let eval0 = evaluate_expression_internal(&n.childs[0], code, temp_ids, manager);
            let eval1 = evaluate_expression_internal(&n.childs[1], code, temp_ids, manager);
            eval_comp_op(&eval0, &eval1, &**op_name, code, temp_ids, manager)
        },
        TokLogOp { ref op_name, .. } => {
            let eval0 = evaluate_expression_internal(&n.childs[0], code, temp_ids, manager);
            
            match &**op_name {
                "and" | "or" => {
                    let eval1 = evaluate_expression_internal(&n.childs[1], code, temp_ids, manager);
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

                    let from_value = evaluate_expression_internal(from, code, temp_ids, manager);
                    let to_value = evaluate_expression_internal(to, code, temp_ids, manager);
                    codegen::function_random(&from_value, &to_value, code, temp_ids)
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
        "neq" => { val0 != val1 },
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