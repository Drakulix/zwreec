//! The `evaluate_expressions` module...


use backend::zcode::zfile::{ZOP, Operand, Variable, Constant, LargeConstant, Zfile, Type};
use frontend::ast::{ASTNode};
use frontend::codegen;
use frontend::codegen::{CodeGenManager};
use frontend::lexer::Token::{TokNumOp, TokCompOp, TokLogOp, TokInt, TokBoolean, TokVariable, TokFunction, TokString, TokUnaryMinus};



pub fn evaluate_expression<'a>(node: &'a ASTNode, code: &mut Vec<ZOP>, mut manager: &mut CodeGenManager<'a>, mut out: &mut Zfile) -> Operand {
    let mut temp_ids = CodeGenManager::new_temp_var_vec();
    evaluate_expression_internal(node, code, &mut temp_ids, manager, &mut out)
}

/// Evaluates an expression node to zCode.
fn evaluate_expression_internal<'a>(node: &'a ASTNode, code: &mut Vec<ZOP>,
        temp_ids: &mut Vec<u8>, mut manager: &mut CodeGenManager<'a>, mut out: &mut Zfile) -> Operand {
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
                "and" | "&&" | "or" | "||" => {
                    let eval1 = evaluate_expression_internal(&n.childs[1], code, temp_ids, manager, &mut out);
                    eval_and_or(&eval0, &eval1, &**op_name, code, temp_ids)
                },
                "not" | "!" => {
                    eval_not(&eval0, code, temp_ids, manager)
                },
                _ => panic!("unhandled op")
            }
        },
        TokUnaryMinus { .. } => {
            let eval = evaluate_expression_internal(&n.childs[0], code, temp_ids, manager, &mut out);
            eval_unary_minus(&eval, code, temp_ids)
        },
        TokInt { ref value, .. } => {
            Operand::new_large_const(*value as i16)
        },
        TokBoolean { ref value, .. } => {
            boolstr_to_const(&**value)
        },
        TokString {ref value, .. } => {
            Operand::new_string_ref(out.write_string(value) as i16)
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

                    let from_value = evaluate_expression_internal(from, code, temp_ids, manager, &mut out);
                    let to_value = evaluate_expression_internal(to, code, temp_ids, manager, &mut out);
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
            if save_var.vartype == Type::String {
                let addr1 = match eval0 {
                    &Operand::StringRef(_) => eval0,
                    &Operand::Var(Variable{id: _, vartype: Type::String}) => eval0,
                    _ => panic!("num_to_str not implemented") // @TODO: would return addr of number converted to str in mem
                };
                let addr2 = match eval1 {
                    &Operand::StringRef(_) => eval1,
                    &Operand::Var(Variable{id: _, vartype: Type::String}) => eval1,
                    _ => panic!("num_to_str not implemented") // @TODO: would return addr of number converted to str in mem
                };
                let len1: Variable = match temp_ids.pop() {
                    Some(var) => Variable::new(var),
                    None      => panic!{"Stack temp_ids is empty, pop wasn't possible."}
                };
                let len2: Variable = match temp_ids.pop() {
                    Some(var) => Variable::new(var),
                    None      => panic!{"Stack temp_ids is empty, pop wasn't possible."}
                };
                let tmp: Variable = match temp_ids.pop() {
                    Some(var) => Variable::new(var),
                    None      => panic!{"Stack temp_ids is empty, pop wasn't possible."}
                };
                let codesnippet = vec![
                    // set to 0 for index access
                    ZOP::StoreVariable{variable: len1.clone(), value: Operand::new_large_const(0)},
                    // read length of string1 which is stored at index 0
                    ZOP::LoadW{array_address: addr1.clone(), index: len1.clone(), variable: len1.clone()},
                    // set to 0 for index access
                    ZOP::StoreVariable{variable: len2.clone(), value: Operand::new_large_const(0)},
                    // read length of string2 which is stored at index 0
                    ZOP::LoadW{array_address: addr2.clone(), index: len2.clone(), variable: len2.clone()},
                    // store new length = len1+len2 in save_var
                    ZOP::StoreVariable{variable: save_var.clone(), value: Operand::new_var(len1.id)},
                    ZOP::Add{operand1: Operand::new_var(len2.id), operand2: Operand::new_var(save_var.id), save_variable: save_var.clone()},
                    ZOP::Inc{variable: save_var.id},  // increase as we will also save the length at first u16
                    ZOP::Call2S{jump_to_label: "malloc".to_string(), arg: Operand::new_var(save_var.id), result: save_var.clone()},
                    // write len1+len2 to len2
                    ZOP::Add{operand1: Operand::new_var(len1.id), operand2: Operand::new_var(len2.id), save_variable: len2.clone()},
                    // set tmp to 0 for array index 0
                    ZOP::StoreVariable{variable: tmp.clone(), value: Operand::new_large_const(0)},
                    // and store len1+len2 in first u16
                    ZOP::StoreW{array_address: Operand::new_var(save_var.id), index: tmp.clone(), variable: len2.clone()},
                    // set tmp to save_var_addr+2
                    ZOP::StoreVariable{variable: tmp.clone(), value: Operand::new_large_const(2)},
                    ZOP::Add{operand1: Operand::new_var(tmp.id), operand2: Operand::new_var(save_var.id), save_variable: tmp.clone()},
                    // strcopy (addr1 to save_var_addr+2)
                    ZOP::CallVNA2{jump_to_label: "strcpy".to_string(), arg1: addr1.clone(), arg2: Operand::new_var(tmp.id)},
                    // set tmp to save_var_addr+2+len1*2
                    ZOP::Add{operand1: Operand::new_var(tmp.id), operand2: Operand::new_var(len1.id), save_variable: tmp.clone()},
                    ZOP::Add{operand1: Operand::new_var(tmp.id), operand2: Operand::new_var(len1.id), save_variable: tmp.clone()},
                    // strcopy (addr2 to save_var_addr+2+len1*2)
                    ZOP::CallVNA2{jump_to_label: "strcpy".to_string(), arg1: addr2.clone(), arg2: Operand::new_var(tmp.id)},
                ];
                for instr in codesnippet {
                    code.push(instr);
                }
                free_var_if_temp(&Operand::new_var(len1.id), temp_ids);
                free_var_if_temp(&Operand::new_var(len2.id), temp_ids);
                free_var_if_temp(&Operand::new_var(tmp.id), temp_ids);
            } else {
                code.push(ZOP::Add{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
            }
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
    let save_var: Variable = match temp_ids.pop() {
        Some(var) => Variable::new(var),
        None      => panic!{"Stack temp_ids is empty, pop wasn't possible."}
    };
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
        let result = if op_name == "or" || op_name == "||" {
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
    if op_name == "or" || op_name == "||" {
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
    let save_var: Variable = match temp_ids.pop() {
        Some(var) => Variable::new(var),
        None      => panic!{"Stack temp_ids is empty, pop wasn't possible."}
    };
    let label = format!("expr_{}", manager.ids_expr.start_next());
    code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: Operand::new_const(0)});
    code.push(ZOP::JG{operand1: eval.clone(), operand2: Operand::new_const(0), jump_to_label: label.to_string()});
    code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: Operand::new_const(1)});
    code.push(ZOP::Label {name: label.to_string()});
    free_var_if_temp(eval, temp_ids);
    Operand::Var(save_var)
}

fn eval_unary_minus(eval: &Operand, code: &mut Vec<ZOP>, temp_ids: &mut Vec<u8>) -> Operand {
    if eval.is_const() {
        let large = match eval { &Operand::LargeConst(_) => { true }, _ => { false } };
        if large {
            return Operand::new_large_const(-eval.const_value());
        } else {
            return Operand::new_const(-eval.const_value() as u8);
        }
    }

    let save_var = match eval {
        &Operand::Var(ref var) => {
            if CodeGenManager::is_temp_var(var) {
                Variable::new(var.id)
            } else {
                if let Some(temp) = temp_ids.pop() {
                    Variable::new(temp)
                } else {
                    panic!{"Stack temp_ids is empty, pop wasn't possible."}
                }
            }
        }, _ => {
            if let Some(temp) = temp_ids.pop() {
                Variable::new(temp)
            } else {
                panic!{"Stack temp_ids is empty, pop wasn't possible."}
            }
        }
    };

    code.push(ZOP::Sub {operand1: Operand::new_const(0), operand2: eval.clone(), save_variable: save_var.clone()});

    Operand::new_var(save_var.id)
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

fn determine_result_type(a: Type, b: Type) -> Type {
    if a == Type::String || b == Type::String {
        Type::String
    } else {
        Type::Integer
    }
}

fn determine_save_var(operand1: &Operand, operand2: &Operand, temp_ids: &mut Vec<u8>) -> Variable {
    let type1 = match operand1 {
        &Operand::Var(ref var) => var.vartype.clone(),
        &Operand::StringRef(_) => Type::String,
        _ => { Type::Integer }
    };
    let type2 = match operand2 {
        &Operand::Var(ref var) => var.vartype.clone(),
        &Operand::StringRef(_) => Type::String,
        _ => { Type::Integer }
    };
    let vartype = determine_result_type(type1, type2);
    match operand1 {
        &Operand::Var(ref var) => {
            if CodeGenManager::is_temp_var(var) {
                return Variable{id: var.id, vartype: vartype};
            }
        }, _ => {}
    };
    match operand2 {
        &Operand::Var(ref var) => {
            if CodeGenManager::is_temp_var(var) {
                return Variable{id: var.id, vartype: vartype};
            }
        }, _ => {}
    };
    if let Some(temp) = temp_ids.pop() {
        return Variable{ id: temp, vartype: vartype };
    } else {
        panic!{"Stack temp_ids is empty, pop wasn't possible."}
    }
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
