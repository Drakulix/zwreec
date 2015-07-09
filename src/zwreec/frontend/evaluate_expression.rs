//! The `evaluate_expressions` module evaluates expressions from
//! the AST and compiles them into zCode.
//!
//! This module takes the root node of an expression and traverses
//! the contained subtree. It analyses if a sub-expression only
//! contains constants and if so, evaluates them while compiling.
//! Otherwise it translates the expressions into zCode. The module uses
//! a finite amount of local variables in zCode to evaluate the
//! expressions. Hence only expressions with limited size are
//! supported.

use backend::zcode::zfile::{ZOP, Operand, Variable, Constant, LargeConstant, Zfile, Type};
use backend::codegen;
use backend::codegen::{CodeGenManager};
use frontend::ast::{ASTNode};
use frontend::lexer::Token;
use frontend::lexer::Token::{TokNumOp, TokCompOp, TokLogOp, TokInt, TokBoolean, TokVariable, TokArrayLength, TokArrayAccess, TokFunction, TokString, TokUnaryMinus};
#[allow(unused_imports)] use config::Config;

#[derive(Debug)]
pub enum EvaluateExpressionError {
    InvalidAST,
    NumericOperatorNeedsTwoArguments { op_name: String, location: (u64, u64) },
    UnhandledToken { token: Token },
    UnsupportedOperator { op_name: String, location: (u64, u64) },
    UnsupportedFunction { name: String, location: (u64, u64) },
    UnsupportedFunctionArgsLen { name: String, location: (u64, u64), expected: u64 },
    UnsupportedFunctionArgType { name: String, index: u64, location: (u64, u64) },
    NoTempIdLeftOnStack,
}

/// This functions evaluates an expression from the AST and returns an `Operand` containing the result.
/// # Arguments
/// `node` is the root node of the expression. Mostly the child of `TokExpression` is what you want to give here.
/// `code` is the vector where the zCode shall be written to.
/// `manager` is the manager from `codegen`. It is required for the symbol table and label ids.
/// `out` is the `ZFile` compiling to. It is required for storing strings.
pub fn evaluate_expression(node: ASTNode, code: &mut Vec<ZOP>, mut manager: &mut CodeGenManager, mut out: &mut Zfile) -> Operand {
    let mut temp_ids = CodeGenManager::new_temp_var_vec();
    evaluate_expression_internal(node, code, &mut temp_ids, manager, &mut out)
}

/// Evaluates an expression node to zCode.
fn evaluate_expression_internal(node: ASTNode, code: &mut Vec<ZOP>,
        temp_ids: &mut Vec<u8>, mut manager: &mut CodeGenManager, mut out: &mut Zfile) -> Operand {
    let n = node.clone().as_default();
    let cfg = manager.cfg;

    match n.category {
        TokNumOp { ref op_name, ref location } => {
            if n.childs.len() != 2 {
                error_panic!(cfg => EvaluateExpressionError::NumericOperatorNeedsTwoArguments { op_name: op_name.clone(), location: location.clone() } );

                // Try error recovery. Ignores operator.
                warn!("Trying to fix expression. This will compile but probably not do what you want.");
                if n.childs.len() >= 1 {
                    return evaluate_expression_internal(n.childs[0].clone(), code, temp_ids, manager, &mut out)
                } else {
                    return Operand::Const(Constant { value: 0 })
                }
            }

            let eval0 = evaluate_expression_internal(n.childs[0].clone(), code, temp_ids, manager, &mut out);
            let eval1 = evaluate_expression_internal(n.childs[1].clone(), code, temp_ids, manager, &mut out);
            eval_num_op(&eval0, &eval1, &**op_name, location.clone(), code, temp_ids, manager)
        },
        TokCompOp { ref op_name, ref location } => {
            if n.childs.len() != 2 {
                error_panic!(cfg => EvaluateExpressionError::NumericOperatorNeedsTwoArguments { op_name: op_name.clone(), location: location.clone() } );

                // Try error recovery. Ignores operator.
                warn!("Trying to fix expression. This will compile but probably not do what you want.");
                if n.childs.len() >= 1 {
                    return evaluate_expression_internal(n.childs[0].clone(), code, temp_ids, manager, &mut out)
                } else {
                    return Operand::BoolConst(Constant { value: 0 })
                }
            }

            let eval0 = evaluate_expression_internal(n.childs[0].clone(), code, temp_ids, manager, &mut out);
            let eval1 = evaluate_expression_internal(n.childs[1].clone(), code, temp_ids, manager, &mut out);
            eval_comp_op(&eval0, &eval1, &**op_name, location.clone(), code, temp_ids, manager)
        },
        TokLogOp { ref op_name, ref location } => {
            let eval0 = evaluate_expression_internal(n.childs[0].clone(), code, temp_ids, manager, &mut out);

            match &**op_name {
                "and" | "&&" | "or" | "||" => {
                    let eval1 = evaluate_expression_internal(n.childs[1].clone(), code, temp_ids, manager, &mut out);
                    eval_and_or(&eval0, &eval1, &**op_name, code, temp_ids)
                },
                "not" | "!" => {
                    eval_not(&eval0, code, temp_ids, manager)
                },
                _ => {
                    error_panic!(cfg => EvaluateExpressionError::UnsupportedOperator { op_name: op_name.clone(), location: location.clone() } );

                    // Try error recovery. Ignores operator.
                    warn!("Trying to fix expression. This will compile but probably not do what you want.");
                    if n.childs.len() >= 1 {
                        return evaluate_expression_internal(n.childs[0].clone(), code, temp_ids, manager, &mut out)
                    } else {
                        return Operand::BoolConst(Constant { value: 0 })
                    }
                }
            }
        },
        TokUnaryMinus { .. } => {
            let eval = evaluate_expression_internal(n.childs[0].clone(), code, temp_ids, manager, &mut out);
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
        TokVariable { name, .. } => {
            Operand::Var(manager.symbol_table.get_and_add_symbol_id(name))
        },
        TokArrayLength { ref name, .. } => {
            let alen: Variable = match temp_ids.pop() {
                Some(var) => Variable::new(var),
                None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
            };
            let zero: Variable = match temp_ids.pop() {
                Some(var) => Variable::new(var),
                None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
            };
            let var = Operand::Var(manager.symbol_table.get_and_add_symbol_id(name));
            code.push(ZOP::StoreVariable{variable: zero.clone(), value: Operand::new_large_const(0)},);
            code.push(ZOP::LoadW{array_address: var, index: zero.clone(), variable: alen.clone()});
            code.push(ZOP::SetVarType{variable: alen.clone(), vartype: Type::Integer});
            temp_ids.push(zero.id);
            Operand::new_var(alen.id)
        },
        TokArrayAccess { ref name, ref index, .. } => {
            let val: Variable = match temp_ids.pop() {
                Some(var) => Variable::new(var),
                None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
            };
            let mem: Variable = match temp_ids.pop() {
                Some(var) => Variable::new(var),
                None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
            };
            let ind: Variable = match temp_ids.pop() {
                Some(var) => Variable::new(var),
                None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
            };
            let var = Operand::Var(manager.symbol_table.get_and_add_symbol_id(name));
            let index = Operand::Var(manager.symbol_table.get_and_add_symbol_id(index));
            code.push(ZOP::Call2S{jump_to_label: "malloc".to_string(), arg: Operand::new_const(2), result: mem.clone()});
            code.push(ZOP::StoreVariable{variable: ind.clone(), value: Operand::new_large_const(0)});
            code.push(ZOP::StoreVariable{variable: val.clone(), value: Operand::new_large_const(1)});
            code.push(ZOP::StoreW{array_address: Operand::new_var(mem.id), index: ind.clone(), variable: val.clone()});
            code.push(ZOP::StoreVariable{variable: val.clone(), value: index.clone()});
            code.push(ZOP::Inc{variable: val.id});
            code.push(ZOP::LoadW{array_address: var, index: val.clone(), variable: val.clone()});
            code.push(ZOP::StoreVariable{variable: ind.clone(), value: Operand::new_large_const(1)});
            code.push(ZOP::StoreW{array_address: Operand::new_var(mem.id), index: ind.clone(), variable: val.clone()});
            code.push(ZOP::SetVarType{variable: mem.clone(), vartype: Type::String});
            temp_ids.push(val.id);
            temp_ids.push(ind.id);
            Operand::new_var(mem.id)
        },
        TokFunction { ref name, ref location } => {
            match &**name {
                "random" => {
                    let args = node.clone().as_default().childs;
                    if args.len() != 2 {
                        let error = EvaluateExpressionError::UnsupportedFunctionArgsLen {
                            name: "random".to_string(), location: location.clone(), expected: 2 };
                        error_panic!(cfg => error);
                        if args.len() <= 1 {
                            return Operand::Const(Constant { value: 0 })
                        } else {
                            warn!("Ignoring the additional arguments.");
                        }
                    }

                    if args[0].clone().as_default().childs.len() != 1 || args[1].clone().as_default().childs.len() != 1 {
                        error_force_panic!(EvaluateExpressionError::InvalidAST);
                    }

                    let from = args[0].clone().as_default().childs[0].clone();
                    let to = args[1].clone().as_default().childs[0].clone();

                    let from_value = evaluate_expression_internal(from, code, temp_ids, manager, &mut out);
                    let to_value = evaluate_expression_internal(to, code, temp_ids, manager, &mut out);
                    codegen::function_random(manager, &from_value, &to_value, code, temp_ids, location.clone())
                },
                "prompt" => { // twee function prompt(message, default) - imitates the JS browser input dialog
                    let args = &node.as_default().childs;
                    if args.len() != 2 {
                        let error = EvaluateExpressionError::UnsupportedFunctionArgsLen {
                            name: "prompt".to_string(), location: location.clone(), expected: 2 };
                        error_panic!(cfg => error);
                        if args.len() <= 1 {
                            return Operand::Const(Constant { value: 0 })
                        } else {
                            warn!("Ignoring the additional arguments.");
                        }
                    }

                    if args[0].as_default().childs.len() != 1 || args[1].as_default().childs.len() != 1 {
                        error_force_panic!(EvaluateExpressionError::InvalidAST);
                    }

                    let message_n = &args[0].as_default().childs[0];
                    let default_n = &args[1].as_default().childs[0];

                    let message = evaluate_expression_internal(message_n, code, temp_ids, manager, &mut out);
                    let default = evaluate_expression_internal(default_n, code, temp_ids, manager, &mut out);
                    let return_var: Variable = match temp_ids.pop() {
                        Some(var) => Variable::new(var),
                        None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
                    };
                    code.push(ZOP::CallVSA2{jump_to_label: "rt_prompt".to_string(), arg1: message.clone(), arg2: default.clone(), result: return_var.clone()});
                    code.push(ZOP::SetVarType{variable: return_var.clone(), vartype: Type::String});
                    Operand::new_var(return_var.id)
                },
                "confirm" => {
                    let state_copy = manager.format_state.clone();
                    let args = &node.as_default().childs;
                    if args.len() != 1 {
                        let error = EvaluateExpressionError::UnsupportedFunctionArgsLen {
                            name: "confirm".to_string(), location: location.clone(), expected: 2 };
                        error_panic!(cfg => error);
                    }
                    if args[0].as_default().childs.len() != 1 {
                        error_force_panic!(EvaluateExpressionError::InvalidAST);
                    }
                    let child = args[0].as_default().childs[0].as_default();
                    let confirm_msg = match child.category {
                        TokString {ref value, .. } => value,
                        _ => error_force_panic!(EvaluateExpressionError::InvalidAST)
                    };

                    let has_confirmed: Variable = match temp_ids.pop() {
                        Some(var) => Variable::new(var),
                        None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
                    };
                    
                    //let confirm_msg = &args[0].as_default().childs[0];
                    //println!("confirm_msg: {:?}", confirm_msg);
                    let if_id = manager.ids_if.start_next();
                    let true_label = format!("true_{}", if_id);
                    let false_label = format!("false_{}", if_id);
                    let repeat_label = format!("repeat_{}", if_id);
                    let end_label = format!("end_{}", if_id);

                    code.push(ZOP::SetTextStyle{bold: true, reverse: state_copy.inverted, monospace: true, italic: state_copy.italic});
                    code.push(ZOP::Label{name: repeat_label.to_string()});
                    code.push(ZOP::PrintOps{text: "------------------------------------------------------------".to_string()});
                    code.push(ZOP::Newline);
                    code.push(ZOP::PrintOps{text: "|  ".to_string()+&confirm_msg.to_string()});
                    code.push(ZOP::Newline);
                    code.push(ZOP::PrintOps{text: "|  Confirm with the 1-key or deny with the 0-key".to_string()});
                    code.push(ZOP::Newline);
                    code.push(ZOP::PrintOps{text: "------------------------------------------------------------".to_string()});
                    code.push(ZOP::Newline);
                    code.push(ZOP::ReadChar{local_var_id: has_confirmed.id});
                    
                    //code.push(ZOP::PrintNumVar{variable: has_confirmed.clone()});
                    code.push(ZOP::JE{operand1: Operand::new_var(has_confirmed.id), operand2: Operand::new_const(48), jump_to_label: false_label.to_string()});
                    code.push(ZOP::JE{operand1: Operand::new_var(has_confirmed.id), operand2: Operand::new_const(49), jump_to_label: true_label.to_string()});
                    code.push(ZOP::PrintOps{text: "Error, this key wasn't possible.".to_string()});
                    code.push(ZOP::Newline);
                    code.push(ZOP::Jump{jump_to_label: repeat_label.to_string()});
                    code.push(ZOP::Label{name: true_label.to_string()});
                    code.push(ZOP::StoreVariable{variable: has_confirmed.clone(), value: Operand::BoolConst(Constant {value: 1})});
                    code.push(ZOP::Jump{jump_to_label: end_label.to_string()});
                    code.push(ZOP::Label{name: false_label.to_string()});
                    code.push(ZOP::StoreVariable{variable: has_confirmed.clone(), value: Operand::BoolConst(Constant {value: 0})});
                    code.push(ZOP::Label{name: end_label.to_string()});
                    code.push(ZOP::SetTextStyle{bold: state_copy.bold, reverse: state_copy.inverted, monospace: state_copy.mono, italic: state_copy.italic});
                    code.push(ZOP::SetVarType{variable: Variable::new(has_confirmed.id), vartype: Type::Bool});
                    Operand::new_var(has_confirmed.id)
                },
                _ => {
                    error_panic!(cfg => EvaluateExpressionError::UnsupportedFunction { name: name.clone(), location: location.clone() });
                    Operand::Const(Constant { value: 0 })
                }
            }
        },
        _ => {
            error_panic!(cfg => EvaluateExpressionError::UnhandledToken { token: n.category.clone() } );
            Operand::Const(Constant { value: 0 })
        }
    }
}

fn eval_num_op(eval0: &Operand, eval1: &Operand, op_name: &str, location: (u64, u64), code: &mut Vec<ZOP>, temp_ids: &mut Vec<u8>, manager: &CodeGenManager) -> Operand {
    if count_constants(eval0, eval1) == 2 {
        return direct_eval_num_op(eval0, eval1, op_name, location, manager);
    }
    let save_var = determine_save_var(eval0, eval1, temp_ids);
    match op_name {
        "+" => {
            let tmp1: Variable = match temp_ids.pop() {
                Some(var) => Variable::new(var),
                None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
            };
            let tmp2: Variable = match temp_ids.pop() {
                Some(var) => Variable::new(var),
                None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
            };
            code.push(ZOP::AddTypes{operand1: eval0.clone(), operand2: eval1.clone(), tmp1: tmp1.clone(), tmp2: tmp2.clone(), save_variable: save_var.clone()});
            free_var_if_temp(&Operand::new_var(tmp1.id), temp_ids);
            free_var_if_temp(&Operand::new_var(tmp2.id), temp_ids);
        },
        "-" => {
            code.push(ZOP::Sub{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
            code.push(ZOP::SetVarType{variable: save_var.clone(), vartype: save_var.vartype.clone()});
        },
        "*" => {
            code.push(ZOP::Mul{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
            code.push(ZOP::SetVarType{variable: save_var.clone(), vartype: save_var.vartype.clone()});
        },
        "/" => {
            code.push(ZOP::Div{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
            code.push(ZOP::SetVarType{variable: save_var.clone(), vartype: save_var.vartype.clone()});
        },
        "%" => {
            code.push(ZOP::Mod{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
            code.push(ZOP::SetVarType{variable: save_var.clone(), vartype: save_var.vartype.clone()});
        },
        _ => {
            error_panic!(manager.cfg => EvaluateExpressionError::UnsupportedOperator { op_name: op_name.to_string(), location: location.clone() })
        }
    };
    free_var_if_both_temp(eval0, eval1, temp_ids);

    Operand::Var(save_var)
}



fn direct_eval_num_op(eval0: &Operand, eval1: &Operand, op_name: &str, location: (u64, u64), manager: &CodeGenManager) -> Operand {
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
        _ => {
            error_panic!(manager.cfg => EvaluateExpressionError::UnsupportedOperator { op_name: op_name.to_string(), location: location.clone() });
            warn!("Returning the first argument of the expression");
            val0
        }
    };
    if out_large {
        Operand::LargeConst(LargeConstant { value: result })
    } else {
        Operand::Const(Constant { value: result as u8 })
    }
}


fn eval_comp_op(eval0: &Operand, eval1: &Operand, op_name: &str, location: (u64, u64), code: &mut Vec<ZOP>,
        temp_ids: &mut Vec<u8>, mut manager: &mut CodeGenManager) -> Operand {
    if count_constants(eval0, eval1) == 2 {
        return direct_eval_comp_op(eval0, eval1, op_name, location.clone(), manager);
    }
    let save_var: Variable = match temp_ids.pop() {
        Some(var) => Variable::new_bool(var),
        None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
    };
    let label_is_bool = format!("expr_{}", manager.ids_expr.start_next());
    let label_is_string = format!("expr_{}", manager.ids_expr.start_next());
    let label = format!("expr_{}", manager.ids_expr.start_next()); // label return
    let const_true = Operand::new_const(1);
    let const_false = Operand::new_const(0);
    // test for type bool and string
    // we only take the first operand's type for this. if it is not a string, but the second, then count both as integers anyway as it make now sense, but does not harm
    match eval0 {
        &Operand::StringRef(_) => { code.push(ZOP::StoreVariable{variable: save_var.clone(), value: Operand::new_const(Type::String as u8)}); },
        &Operand::Var(ref var) => { code.push(ZOP::GetVarType{variable: var.clone(), result: save_var.clone()}); },
        &Operand::BoolConst(_) => { code.push(ZOP::StoreVariable{variable: save_var.clone(), value: Operand::new_const(Type::Bool as u8)}); },
        _ => { code.push(ZOP::StoreVariable{variable: save_var.clone(), value: Operand::new_const(Type::Integer as u8)}); }
    };
    code.push(ZOP::JE{operand1: Operand::new_var(save_var.id), operand2: Operand::new_const(Type::String as u8), jump_to_label: label_is_string.to_string()});
    code.push(ZOP::JE{operand1: Operand::new_var(save_var.id), operand2: Operand::new_const(Type::Bool as u8), jump_to_label: label_is_bool.to_string()});
    // compare as numbers
    match op_name {
        "is" | "==" | "eq" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
            code.push(ZOP::JE{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
        },
        "!=" | "neq" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
            code.push(ZOP::JE{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
        },
        "<" | "lt" =>  {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone() });
            code.push(ZOP::JL{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
        },
        "<=" | "lte" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
            code.push(ZOP::JG{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
        },
        ">=" | "gte" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
            code.push(ZOP::JL{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
        },
        ">" | "gt" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
            code.push(ZOP::JG{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
        },
        _ => {
            error_panic!(manager.cfg => EvaluateExpressionError::UnsupportedOperator { op_name: op_name.to_string(), location: location.clone() });
            warn!("Assuming 'false' as the result");
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone() });
        }
    };
    code.push(ZOP::Jump{jump_to_label: label.to_string()});
    code.push(ZOP::Label {name: label_is_bool.to_string()});
    // @TODO: compare as bool regarding that e.g. -31 should be seen as true
    match op_name {
        "is" | "==" | "eq" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
            code.push(ZOP::JE{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
        },
        "!=" | "neq" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
            code.push(ZOP::JE{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
        },
        "<" | "lt" =>  {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone() });
            code.push(ZOP::JL{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
        },
        "<=" | "lte" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
            code.push(ZOP::JG{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
        },
        ">=" | "gte" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
            code.push(ZOP::JL{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
        },
        ">" | "gt" => {
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_true.clone()});
            code.push(ZOP::JG{operand1: eval0.clone(), operand2: eval1.clone(), jump_to_label: label.to_string()});
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone()});
        },
        _ => {
            error_panic!(manager.cfg => EvaluateExpressionError::UnsupportedOperator { op_name: op_name.to_string(), location: location.clone() });
            warn!("Assuming 'false' as the result");
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false.clone() });
        }
    };
    code.push(ZOP::Jump{jump_to_label: label.to_string()});
    code.push(ZOP::Label {name: label_is_string.to_string()});
    // compare as strings
    code.push(ZOP::CallVSA2{jump_to_label: "strcmp".to_string(), arg1: eval0.clone(), arg2: eval1.clone(), result: save_var.clone()},);
    match op_name {
        "is" | "==" | "eq" => { // we only want true if the result is not 0
            // so first we make 0 to ffff while -1 and 1 will lose their last bit. and then we AND the last bit
            code.push(ZOP::Not{operand: Operand::new_var(save_var.id), result: save_var.clone()});
            code.push(ZOP::And{operand1: Operand::new_var(save_var.id), operand2: Operand::new_large_const(1i16), save_variable: save_var.clone()});
        },
        "!=" | "neq" => {},  // we can leave the result as it is
        "<" | "lt" =>  {  // we want only true if the result was -1,
            // so for 0 and 1 we AND with every bit on except the last bit off which is then gone
            // and the result is 0. for -1 this does not make it 0 as there are more bits left
            code.push(ZOP::And{operand1: Operand::new_var(save_var.id), operand2: Operand::new_large_const(-2i16), save_variable: save_var.clone()});
        },
        "<=" | "lte" => {  // we do not want true for 1, so we make 0 out of it by decreasing
            code.push(ZOP::Dec{variable: save_var.id});
        },
        ">=" | "gte" => {  // we do not want true for -1, so we make 0 out of it by increasing
            code.push(ZOP::Inc{variable: save_var.id});
        },
        ">" | "gt" => { // we want only true if the result was 1. so we increase it to 2 and AND with 2,
            // so only the second bit survives
            code.push(ZOP::Inc{variable: save_var.id});
            code.push(ZOP::And{operand1: Operand::new_var(save_var.id), operand2: Operand::new_large_const(2), save_variable: save_var.clone()});
        },
        _ => {
            error_panic!(manager.cfg => EvaluateExpressionError::UnsupportedOperator { op_name: op_name.to_string(), location: location.clone() });
            warn!("Assuming 'false' as the result");
            code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: const_false });
        }
    };
    code.push(ZOP::Label {name: label.to_string()});
    code.push(ZOP::SetVarType{variable: save_var.clone(), vartype: Type::Bool});
    free_var_if_temp(eval0, temp_ids);
    free_var_if_temp(eval1, temp_ids);
    Operand::Var(save_var)
}

/// Directly evaluates the given compare operation.
/// Both operands must be constants.
fn direct_eval_comp_op(eval0: &Operand, eval1: &Operand, op_name: &str, location: (u64, u64), manager: &CodeGenManager) -> Operand {
    let val0 = eval0.const_value();
    let val1 = eval1.const_value();
    let result = match op_name {
        "is" | "==" | "eq" => { val0 == val1 },
        "!=" | "neq" => { val0 != val1 },
        "<" | "lt" =>  { val0 < val1 },
        "<=" | "lte" => { val0 <= val1 },
        ">=" | "gte" => { val0 >= val1 },
        ">" | "gt" => { val0 > val1 },
        _ => {
            error_panic!(manager.cfg => EvaluateExpressionError::UnsupportedOperator { op_name: op_name.to_string(), location: location.clone() });
            warn!("Assuming 'false' as the result");
            false
        }
    };
    if result {
        Operand::BoolConst(Constant {value: 1})
    } else {
        Operand::BoolConst(Constant {value: 0})
    }
}


fn eval_and_or(eval0: &Operand, eval1: &Operand, op_name: &str, code: &mut Vec<ZOP>,
        temp_ids: &mut Vec<u8>) -> Operand {
    if count_constants(&eval0, &eval1) == 2 {
        let val0 = eval0.const_value();
        let val1 = eval1.const_value();
        let result = if op_name == "or" || op_name == "||" {
                val0 | val1
            } else {
                val0 & val1
            };
        return Operand::BoolConst(Constant { value: if result == 0 { 0 } else { 1 } });
    }

    let save_var = determine_save_var(eval0, eval1, temp_ids);
    if op_name == "or" || op_name == "||" {
        code.push(ZOP::Or{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
    } else {
        code.push(ZOP::And{operand1: eval0.clone(), operand2: eval1.clone(), save_variable: save_var.clone()});
    }
    code.push(ZOP::SetVarType{variable: save_var.clone(), vartype: Type::Bool});
    free_var_if_both_temp(eval0, eval1, temp_ids);
    Operand::new_var_bool(save_var.id)
}


fn eval_not(eval: &Operand, code: &mut Vec<ZOP>,
        temp_ids: &mut Vec<u8>, mut manager: &mut CodeGenManager) -> Operand {
    if eval.is_const() {
        let val = eval.const_value();
        let result: u8 = if val != 0 { 0 } else { 1 };
        return Operand::BoolConst(Constant { value: result });
    }
    let save_var: Variable = match temp_ids.pop() {
        Some(var) => Variable::new_bool(var),
        None      => error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
    };
    let label = format!("expr_{}", manager.ids_expr.start_next());
    code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: Operand::new_const(0)});
    code.push(ZOP::JNE{operand1: eval.clone(), operand2: Operand::new_const(0), jump_to_label: label.to_string()});
    code.push(ZOP::StoreVariable{ variable: save_var.clone(), value: Operand::new_const(1)});
    code.push(ZOP::Label {name: label.to_string()});
    code.push(ZOP::SetVarType{variable: save_var.clone(), vartype: save_var.vartype.clone()});
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
                    error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
                }
            }
        }, _ => {
            if let Some(temp) = temp_ids.pop() {
                Variable::new(temp)
            } else {
                error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
            }
        }
    };

    code.push(ZOP::Sub {operand1: Operand::new_const(0), operand2: eval.clone(), save_variable: save_var.clone()});
    code.push(ZOP::SetVarType{variable: save_var.clone(), vartype: Type::Integer});

    Operand::new_var(save_var.id)
}

/// Checks if both operands are temporary variables. If so, the id of the second
/// variable is pushed onto the temp_ids stack for reuse.
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

/// Checks if the given operand is a temporary variable and if so,
/// pushes the id onto the temp_ids stack for reuse.
fn free_var_if_temp (operand: &Operand, temp_ids: &mut Vec<u8>) {
    match operand {
        &Operand::Var(ref var) => {
            if CodeGenManager::is_temp_var(var){
                temp_ids.push(var.id);
            }
        }, _ => {}
    }
}

/// Determine what type an expression becomes considering
/// the operand types a and b.
fn determine_result_type(a: Type, b: Type) -> Type {
    if a == Type::String || b == Type::String {
        Type::String
    } else {
        Type::Integer
    }
}

/// Determines a variable where the result of an operation on operand1 and operand2 should
/// be saved. if for example both operands are temporary ids, then one of them can be used
/// to store the result. Otherwise a new temp_id will be popped from the stack.
fn determine_save_var(operand1: &Operand, operand2: &Operand, temp_ids: &mut Vec<u8>) -> Variable {
    let type1 = match operand1 {
        &Operand::Var(ref var) => var.vartype.clone(),
        &Operand::StringRef(_) => Type::String,
        &Operand::BoolConst(_) => Type::Bool,
        _ => { Type::Integer }
    };
    let type2 = match operand2 {
        &Operand::Var(ref var) => var.vartype.clone(),
        &Operand::StringRef(_) => Type::String,
        &Operand::BoolConst(_) => Type::Bool,
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
        error_force_panic!(EvaluateExpressionError::NoTempIdLeftOnStack)
    }
}

/// Returns the number of constants, checking operand1 and operand2.
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

/// Converts a boolean string to an integer constant operand
fn boolstr_to_const(string: &str) -> Operand {
    match string {
        "true" => Operand::BoolConst(Constant { value: 1 }),
        _ => Operand::BoolConst(Constant { value: 0 })
    }
}


#[test]
fn test_and_or(){
    let mut vec2: Vec<ZOP> = Vec::new();
    let mut vec: Vec<u8> = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(10);
    assert_eq!(eval_and_or(&Operand::new_large_const(0), &Operand::new_large_const(1), "or", &mut vec2, &mut vec).const_value(),1 as i16);
    assert_eq!(eval_and_or(&Operand::new_large_const(0), &Operand::new_large_const(1), "and", &mut vec2, &mut vec).const_value(),0 as i16);
    assert_eq!(eval_and_or(&Operand::new_large_const(0), &Operand::new_large_const(0), "or", &mut vec2, &mut vec).const_value(),0 as i16);
    assert_eq!(eval_and_or(&Operand::new_large_const(1), &Operand::new_large_const(1), "and", &mut vec2, &mut vec).const_value(),1 as i16);
}

#[test]
fn test_eval_not(){
    let cfg = Config::default_config();
    let mut manager = CodeGenManager::new(&cfg);
    let mut vec2: Vec<ZOP> = Vec::new();
    let mut vec: Vec<u8> = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(10);
    assert_eq!(eval_not(&Operand::new_large_const(10), &mut vec2, &mut vec, &mut manager).const_value(),0);
    assert_eq!(eval_not(&Operand::new_const(0), &mut vec2, &mut vec, &mut manager).const_value(),1);
}

#[test]
fn test_eval_unary_minus(){
    let mut vec2: Vec<ZOP> = Vec::new();
    let mut vec: Vec<u8> = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(10);
    assert_eq!(eval_unary_minus(&Operand::new_large_const(10), &mut vec2, &mut vec).const_value(),-10);
    assert_eq!(eval_unary_minus(&Operand::new_const(10), &mut vec2, &mut vec).const_value(),246);
}

#[test]
fn test_determine_save_var (){
    let mut vec: Vec<u8> = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    let var = determine_save_var(&Operand::new_var(10), &Operand::new_var(10), &mut vec);
    assert_eq!(var.id,10);
    assert_eq!(var.vartype,Type::Integer);
}

#[test]
fn test_count_constants(){
    assert_eq!(count_constants(&Operand::new_large_const(10),&Operand::new_large_const(10)),2);
    assert_eq!(count_constants(&Operand::new_var(10),&Operand::new_large_const(10)),1);
    assert_eq!(count_constants(&Operand::new_large_const(10),&Operand::new_var(10)),1);
    assert_eq!(count_constants(&Operand::new_var(10),&Operand::new_var(10)),0);
}

#[test]
fn test_boolstr_to_const(){
    assert_eq!(boolstr_to_const("true").const_value(),1);
    assert_eq!(boolstr_to_const("false").const_value(),0);
}

#[test]
fn test_direct_eval_num_op(){
    let cfg = Config::default_config();
    let manager = CodeGenManager::new(&cfg);
    assert_eq!(direct_eval_num_op(&Operand::new_large_const(10), &Operand::new_large_const(20), "+", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),30 as i16);
    assert_eq!(direct_eval_num_op(&Operand::new_large_const(66), &Operand::new_large_const(74), "-", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),-8 as i16);
    assert_eq!(direct_eval_num_op(&Operand::new_large_const(45), &Operand::new_large_const(10), "*", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),450 as i16);
    assert_eq!(direct_eval_num_op(&Operand::new_large_const(99), &Operand::new_large_const(3), "/", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),33 as i16);
    assert_eq!(direct_eval_num_op(&Operand::new_large_const(90), &Operand::new_large_const(2), "%", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
}

#[test]
fn test_direct_eval_comp_op(){
    let cfg = Config::default_config();
    let manager = CodeGenManager::new(&cfg);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(20), &Operand::new_large_const(20), "is", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(15), &Operand::new_large_const(15), "==", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(4), &Operand::new_large_const(4), "eq", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(66), &Operand::new_large_const(74), "neq", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(2), &Operand::new_large_const(10), "<", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(5), &Operand::new_large_const(6), "lt", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(5), &Operand::new_large_const(5), "<=", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(2), &Operand::new_large_const(5), "lte", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(6), &Operand::new_large_const(6), ">=", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(6), &Operand::new_large_const(5), "gte", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(4), &Operand::new_large_const(3), ">", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(1), &Operand::new_large_const(0), "gt", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),1 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(0), &Operand::new_large_const(20), "is", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(1), &Operand::new_large_const(15), "==", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(43), &Operand::new_large_const(4), "eq", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(74), &Operand::new_large_const(74), "neq", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(12), &Operand::new_large_const(10), "<", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(15), &Operand::new_large_const(6), "lt", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(6), &Operand::new_large_const(5), "<=", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(7), &Operand::new_large_const(5), "lte", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(5), &Operand::new_large_const(6), ">=", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(3), &Operand::new_large_const(5), "gte", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(2), &Operand::new_large_const(3), ">", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
    assert_eq!(direct_eval_comp_op(&Operand::new_large_const(0), &Operand::new_large_const(0), "gt", (0x0000000000000000, 0x0000000000000000), &manager).const_value(),0 as i16);
}
