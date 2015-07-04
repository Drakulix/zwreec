//! contains most of the op-codes (only the opcode who usees jumps or labels)
//! are still in zfile

pub use super::zfile::ArgType;
pub use super::zfile::JumpType;
pub use super::zfile::Zjump;
pub use super::zfile::Zfile;
pub use super::zfile::{ Operand, Variable, Constant, LargeConstant };

/// clears spcified window
pub fn op_erase_window(value: i8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x0d, args);

    // signed to unsigned value
    write_u16(value as u16, &mut bytes);
    bytes
}


/// calls a routine (the address is stored in a variable)
pub fn op_call_1n_var(variable: u8) -> Vec<u8> {
    let mut bytes = op_1(0x0f, ArgType::Variable);
    bytes.push(variable);
    bytes
}


/// stores a value to an array
/// stores the value of variable to the address in: array_address + 2*index
pub fn op_storew(array_address: &Operand, index: &Variable, variable: &Variable) -> Vec<u8> {
    // assert!(array_address > 0, "not allowed array-address, becouse in _some_ interpreters (for example zoom) it crahs. -.-");
    let args: Vec<ArgType> = vec![arg_type(&array_address), ArgType::Variable, ArgType::Variable, ArgType::Nothing];
    let mut bytes = op_var(0x01, args);

    // array address
    write_argument(array_address, &mut bytes);

    // array index
    bytes.push(index.id);

    // value
    bytes.push(variable.id);
    bytes
}


/// stores a value to an array
/// stores the value of variable to the address in: array_address + index
pub fn op_storeb(array_address: &Operand, index: &Variable, variable: &Variable) -> Vec<u8> {
    // assert!(array_address > 0, "not allowed array-address, becouse in _some_ interpreters (for example zoom) it crahs. -.-");
    let args: Vec<ArgType> = vec![arg_type(&array_address), ArgType::Variable, ArgType::Variable, ArgType::Nothing];
    let mut bytes = op_var(0x02, args);

    // array address
    write_argument(array_address, &mut bytes);

    // array index
    bytes.push(index.id);

    // value
    bytes.push(variable.id);
    bytes
}

/// stores a value to an array
/// stores the value of operand to the address in: array_address + index
pub fn op_storeboperand(array_address: &Operand, index: &Operand, operand: &Operand) -> Vec<u8> {
    // assert!(array_address > 0, "not allowed array-address, becouse in _some_ interpreters (for example zoom) it crahs. -.-");
    let args: Vec<ArgType> = vec![arg_type(&array_address), arg_type(&index), arg_type(&operand), ArgType::Nothing];
    let mut bytes = op_var(0x02, args);

    // array address
    write_argument(array_address, &mut bytes);

    // array index
    write_argument(index, &mut bytes);

    // value
    write_argument(operand, &mut bytes);
    bytes
}

/// loads a byte from an array in a variable
/// loadb is an 2op, BUT with 3 ops -.-
pub fn op_loadb(array_address: &Operand, index: &Operand, variable: &Variable) -> Vec<u8> {
    let mut bytes = op_2(0x10, vec![arg_type(&array_address), arg_type(&index)]);

    // array address
    write_argument(array_address, &mut bytes);
    // array index
    write_argument(index, &mut bytes);

    // variable
    bytes.push(variable.id);
    bytes
}


/// loads a word from an array in a variable
/// loadw is an 2op, BUT with 3 ops -.-
pub fn op_loadw(array_address: &Operand, index: &Variable, variable: &Variable) -> Vec<u8> {
    let mut bytes = op_2(0x0f, vec![arg_type(&array_address), ArgType::Variable]);

    // array address
    write_argument(array_address, &mut bytes);
    // array index
    bytes.push(index.id);

    // variable
    bytes.push(variable.id);
    bytes
}


/// reads keys from the keyboard and saves the asci-value in local_var_id
/// read_char is VAROP
pub fn op_read_char(local_var_id: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x16, args);

    // write argument value
    bytes.push(0x01);

    // write varible id
    bytes.push(local_var_id);
    bytes
}


/// set the style of the text
pub fn op_set_text_style(bold: bool, reverse: bool, monospace: bool, italic: bool) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x11, args);

    let mut style_byte : u8;
    style_byte = 0x00;
    if bold {
        style_byte |=0x02
    }
     if reverse {
        style_byte |=0x01
    }
     if monospace {
        style_byte |=0x08
    }
     if italic {
        style_byte |=0x04
    }
    bytes.push(style_byte);
    bytes
}


/// Prints the value of a variable (only ints a possibe)
pub fn op_print_num_var(variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x06, args);
    bytes.push(variable.id);
    bytes
}


/// pulls an value off the stack to a variable
/// SmallConst because pull takes an reference to a variable
pub fn op_pull(variable: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x09, args);
    bytes.push(variable);
    bytes
}


/// calculates a random numer from 1 to range
pub fn op_random(range: &Operand, variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(range), ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x07, args);
    write_argument(range, &mut bytes);
    bytes.push(variable.id);
    bytes
}


/// pushs an u16 value (for example an address) on the stack
pub fn op_push_u16(value: u16) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x08, args);
    write_u16(value, &mut bytes);
    bytes
}

/// pushs a variable on the stack
pub fn op_push_var(variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x08, args);
    bytes.push(variable.id);
    bytes
}


/// sets the colors of the foreground (font) and background (but with variables
pub fn op_set_color_var(foreground: u8, background: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::Variable];
    let mut bytes = op_2(0x1b, args);
    bytes.push(foreground);
    bytes.push(background);
    bytes
}


/// sets the colors of the foreground (font) and background
pub fn op_set_color(foreground: u8, background: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::SmallConst];
    let mut bytes = op_2(0x1b, args);
    bytes.push(foreground);
    bytes.push(background);
    bytes
}


/// prints string at given packed address (which is then multiplied by 8 the zmachine to be the real address)
pub fn op_print_paddr(address: &Operand) -> Vec<u8> {
   let mut bytes = op_1(0x0D, arg_type(&address));
   write_argument(address, &mut bytes);
   bytes
}

/// prints string at given adress
pub fn op_print_addr(address: &Operand) -> Vec<u8> {
   let mut bytes = op_1(0x07, arg_type(&address));
   write_argument(address, &mut bytes);
   bytes
}

/// returns a LargeConst
pub fn op_ret(value: &Operand) -> Vec<u8> {
    let mut bytes = op_1(0x0b, arg_type(&value));
    write_argument(value, &mut bytes);
    bytes
}

/// saves an operand to the variable
pub fn op_store_var(variable: &Variable, value: &Operand) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Reference, arg_type(&value)];
    let mut bytes = op_2(0x0d, args);
    bytes.push(variable.id);
    write_argument(value, &mut bytes);
    bytes
}

/// saves an operand to the variable id which is given as operand
pub fn op_store_var_id(variable: &Variable, value: &Operand) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, arg_type(&value)];
    let mut bytes = op_2(0x0d, args);
    bytes.push(variable.id);
    write_argument(value, &mut bytes);
    bytes
}

/// bitwise or
/// save_variable = operand1 | operand2
pub fn op_or(operand1: &Operand, operand2: &Operand, save_variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(operand1), arg_type(operand2)];
    let mut bytes = op_2(0x08, args);
    write_argument(operand1, &mut bytes);
    write_argument(operand2, &mut bytes);
    bytes.push(save_variable.id);
    bytes
}

/// bitwise and
/// save_variable = operand1 & operand2
pub fn op_and(operand1: &Operand, operand2: &Operand, save_variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(operand1), arg_type(operand2)];
    let mut bytes = op_2(0x09, args);
    write_argument(operand1, &mut bytes);
    write_argument(operand2, &mut bytes);
    bytes.push(save_variable.id);
    bytes
}

/// bitwise NOT
pub fn op_not(arg: &Operand, variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(arg), ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x18, args);
    write_argument(arg, &mut bytes);
    bytes.push(variable.id);
    bytes
}


/// subtraktion
/// save_variable = operand1 - operand2
pub fn op_sub(operand1: &Operand, operand2: &Operand, save_variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(operand1), arg_type(operand2)];
    let mut bytes = op_2(0x15, args);
    write_argument(operand1, &mut bytes);
    write_argument(operand2, &mut bytes);
    bytes.push(save_variable.id);
    bytes
}

/// addition
/// save_variable = operand1 + operand2
pub fn op_add(operand1: &Operand, operand2: &Operand, save_variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(operand1), arg_type(operand2)];
    let mut bytes = op_2(0x14, args);
    write_argument(operand1, &mut bytes);
    write_argument(operand2, &mut bytes);
    bytes.push(save_variable.id);
    bytes
}

/// multiplikation
/// save_variable = operand1 * operand2
pub fn op_mul(operand1: &Operand, operand2: &Operand, save_variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(operand1), arg_type(operand2)];
    let mut bytes = op_2(0x16, args);
    write_argument(operand1, &mut bytes);
    write_argument(operand2, &mut bytes);
    bytes.push(save_variable.id);
    bytes
}

/// division
/// save_variable = operand1 / operand2
pub fn op_div(operand1: &Operand, operand2: &Operand, save_variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(operand1), arg_type(operand2)];
    let mut bytes = op_2(0x17, args);
    write_argument(operand1, &mut bytes);
    write_argument(operand2, &mut bytes);
    bytes.push(save_variable.id);
    bytes
}

/// modulo
/// save_variable = operand1 % operand2
pub fn op_mod(operand1: &Operand, operand2: &Operand, save_variable: &Variable) -> Vec<u8> {
    let args: Vec<ArgType> = vec![arg_type(operand1), arg_type(operand2)];
    let mut bytes = op_2(0x18, args);
    write_argument(operand1, &mut bytes);
    write_argument(operand2, &mut bytes);
    bytes.push(save_variable.id);
    bytes
}

/// decrements the value of the variable
pub fn op_dec(variable: u8) -> Vec<u8> {
    let mut bytes = op_1(0x06, ArgType::Reference);
    bytes.push(variable);
    bytes
}

/// increments the value of the variable
pub fn op_inc( variable: u8) -> Vec<u8> {
    let mut bytes = op_1(0x05, ArgType::Reference);
    bytes.push(variable);
    bytes
}

/// adds a newline
pub fn op_newline() -> Vec<u8> {
    op_0(0x0b)
}

/// quits the zcode programm immidently
pub fn quit() -> Vec<u8> {
    op_0(0x0a)
}

/// op-codes with 0 operators
/// $b0 -- $bf  short     0OP
pub fn op_0(value: u8) -> Vec<u8> {
    let byte = value | 0xb0;
    vec![byte]
}


/// op-codes with variable operators (4 are possible)
/// $e0 -- $ff  variable  VAR     (operand types in next byte(s))
pub fn op_var(value: u8, arg_types: Vec<ArgType>) -> Vec<u8> {
	let mut ret = Vec::new();
	ret.push(value | 0xe0);
    ret.push(encode_variable_arguments(arg_types));
    ret
}


/// op-codes with 1 operator
/// $80 -- $8f  short     1OP     large constant
/// $90 -- $9f  short     1OP     small constant
/// $a0 -- $af  short     1OP     variable
pub fn op_1( value: u8, arg_type: ArgType) -> Vec<u8> {
    let byte: u8 = match arg_type {
        ArgType::Reference  => 0x90 | value,  // same as SmallConst
        ArgType::Variable   => 0xa0 | value,
        ArgType::SmallConst => 0x90 | value,
        ArgType::LargeConst => 0x80 | value,
        _                   => panic!("no possible 1OP")
    };

    vec![byte]
}

/// $00 -- $1f  long      2OP     small constant, small constant
/// $20 -- $3f  long      2OP     small constant, variable
/// $40 -- $5f  long      2OP     variable, small constant
/// $60 -- $7f  long      2OP     variable, variable
///
/// $c0 -- $df  variable  2OP     (operand types in next byte)
/// not handled here: $be  extended opcode given in next byte
pub fn op_2( value: u8, arg_types: Vec<ArgType>) -> Vec<u8> {
    let mut byte: u8 = 0x00;
    let mut is_variable: bool = false;
    let mut ret = Vec::new();
    for (i, arg_type) in arg_types.iter().enumerate() {
        let shift: u8 = 6 - i as u8;
        match arg_type {
            &ArgType::SmallConst => byte |= 0x00 << shift,
            &ArgType::Variable   => byte |= 0x01 << shift,
            &ArgType::Reference  => byte |= 0x00 << shift,
            &ArgType::LargeConst => is_variable = true,
            _                    => panic!("no possible 2OP")
        }
    }

    if is_variable {
        let mut byte: u8 = 0xc0 | value;
        byte = byte | value;
        ret.push(byte);

        let mut byte2 = encode_variable_arguments(arg_types);
        byte2 = byte2 | 0xf;
        ret.push(byte2)
    } else {
        byte = byte | value;
        ret.push(byte)
    }
    ret
}

pub fn encode_variable_arguments( arg_types: Vec<ArgType>) -> u8 {
    let mut byte: u8 = 0x00;
    for (i, arg_type) in arg_types.iter().enumerate() {
        let shift: u8 = 6 - 2 * i as u8;
        match arg_type {
            &ArgType::LargeConst => byte |= 0x00 << shift,
            &ArgType::SmallConst => byte |= 0x01 << shift,
            &ArgType::Variable   => byte |= 0x02 << shift,
            &ArgType::Nothing    => byte |= 0x03 << shift,
            &ArgType::Reference  => byte |= 0x01 << shift,
            //_                    => panic!("no possible varOP")
        }
    }

    byte
 }


pub fn arg_type(operand: &Operand) -> ArgType {
    match operand {
        &Operand::Var(_) => ArgType::Variable,
        &Operand::Const(_) => ArgType::SmallConst,
        &Operand::BoolConst(_) => ArgType::SmallConst,
        &Operand::LargeConst(_) => ArgType::LargeConst,
        &Operand::StringRef(_) => ArgType::LargeConst,
    }
}

pub fn write_argument(operand: &Operand, v: &mut Vec<u8>){
    match operand {
        &Operand::Var(ref var)=> v.push(var.id),
        &Operand::Const(ref constant) => v.push(constant.value),
        &Operand::BoolConst(ref constant) => v.push(constant.value),
        &Operand::LargeConst(ref constant) => write_i16(constant.value, v),
        &Operand::StringRef(ref constant) => write_i16(constant.value, v),
    };
}

///writes u16 to a vec<u8>
pub fn write_u16(value: u16, v: &mut Vec<u8>) {
        v.push((value >> 8) as u8);
        v.push((value & 0xff) as u8);
}


///writes u16 to a vec<u8>
pub fn write_i16(value: i16, v: &mut Vec<u8>) {
        v.push((value >> 8) as u8);
        v.push((value & 0xff) as u8);
}

