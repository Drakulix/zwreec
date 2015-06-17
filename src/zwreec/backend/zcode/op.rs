pub use super::zfile::ArgType;
pub use super::zfile::JumpType;
pub use super::zfile::Zjump;
pub use super::zfile::Zfile;








/// reads keys from the keyboard and saves the asci-value in local_var_id
/// read_char is VAROP
pub fn op_read_char_timer(local_var_id: u8, timer: u8, routine: &str, zf: &mut Zfile) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::SmallConst, ArgType::LargeConst, ArgType::Nothing];
    let mut bytes = op_var(0x16, args);

    // write argument value
    bytes.push(0x00);

    // write timer
    bytes.push(timer);

    // writes routine
    zf.add_jump(routine.to_string(), JumpType::Routine);

    // write varible id
    bytes.push(local_var_id);
    bytes
}


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
    //self.add_jump(jump_to_label.to_string(), JumpType::Routine);
    bytes.push(variable);
    bytes
}


/// stores a value to an array
/// stores the value of variable to the address in: array_address + 2*index
pub fn op_storew(array_address: u16, index: u8, variable: u8, object_addr: u16) -> Vec<u8> {
    assert!(array_address > 0, "not allowed array-address, becouse in _some_ interpreters (for example zoom) it crahs. -.-");
    let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::Variable, ArgType::Variable, ArgType::Nothing];
    let mut bytes = op_var(0x01, args);

    // array address
    write_u16(object_addr + array_address, &mut bytes);

    // array index
    bytes.push(index);

    // value
    bytes.push(variable);
    bytes
}


/// loads a word from an array in a variable
/// loadw is an 2op, BUT with 3 ops -.-
pub fn op_loadw(array_address: u16, index: u8, variable: u8, object_addr: u16) -> Vec<u8> {
    let mut bytes = op_2(0x0f, vec![ArgType::LargeConst, ArgType::Variable]);

    // array address
    write_u16(object_addr + array_address, &mut bytes);

    // array index
    bytes.push(index);

    // variable
    bytes.push(variable);
    bytes
}


/// reads keys from the keyboard and saves the asci-value in local_var_id
/// read_char is VAROP
pub fn op_read_char(local_var_id: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x16, args);

    // write argument value
    bytes.push(0x00);

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
pub fn op_print_num_var(variable: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x06, args);
    bytes.push(variable);
    bytes
}


/// pulls an value off the stack to an variable
/// SmallConst becouse pull takes an reference to an variable
pub fn op_pull(variable: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x09, args);
    bytes.push(variable);
    bytes
}


/// calculates a random numer from 1 to range
pub fn op_random(range: u8, variable: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x07, args);
    bytes.push(range);
    bytes.push(variable);
    bytes
}


/// pushs an u16 value (for example an address) on the stack
pub fn op_push_u16(value: u16) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    let mut bytes = op_var(0x08, args);
    write_u16(value, &mut bytes);
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


/// prints string at given packet adress TODO: needs testing
pub fn op_print_paddr(address: u8) -> Vec<u8> {
   let mut bytes = op_1(0x0D, ArgType::Variable);
   bytes.push(address);
   bytes
}


/// prints string at given adress TODO: needs testing
pub fn op_print_addr(address: u8) -> Vec<u8> {
   let mut bytes = op_1(0x07, ArgType::Variable);
   bytes.push(address);
   bytes
}


/// returns a SmallConst
pub fn op_ret(value: u8) -> Vec<u8> {
    let mut bytes = op_1(0x0b, ArgType::SmallConst);
    bytes.push(value);
    bytes
}


// saves an u16 to the variable
pub fn op_store_u16(variable: u8, value: u16) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Reference, ArgType::LargeConst];
    let mut bytes = op_2(0x0d, args);
    bytes.push(variable);
    write_u16(value, &mut bytes);
    bytes
}


// saves an u8 to the variable
pub fn op_store_u8(variable: u8, value: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Reference, ArgType::SmallConst];
    let mut bytes = op_2(0x0d, args);
    bytes.push(variable);
    bytes.push(value);
    bytes
}


/// subtraktion
/// variable2 = variable1 - sub_const
pub fn op_sub(variable1: u8, sub_const: i16, variable2: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::LargeConst];
    let mut bytes = op_2(0x15, args);
    bytes.push(variable1);
	write_i16(sub_const, &mut bytes);
    bytes.push(variable2);
    bytes
}


/// addition
/// variable2 = variable1 + sub_const
pub fn op_add(variable1: u8, add_const: i16, variable2: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::LargeConst];
    let mut bytes = op_2(0x14, args);
    bytes.push(variable1);
	write_i16(add_const, &mut bytes);
    bytes.push(variable2);
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

pub fn op_newline() -> Vec<u8> {
    op_0(0x0b)
}

pub fn quit() -> Vec<u8> {
    op_0(0x0a)
}

/// op-codes with 0 operators
fn op_0(value: u8) -> Vec<u8> {
    let byte = value | 0xb0;
    vec![byte]
}


/// op-codes with variable operators (4 are possible)
fn op_var(value: u8, arg_types: Vec<ArgType>) -> Vec<u8> {
	let mut ret = Vec::new();
	ret.push(value | 0xe0);
    ret.push(encode_variable_arguments(arg_types));
    ret
}


/// op-codes with 1 operator
fn op_1( value: u8, arg_type: ArgType) -> Vec<u8> {
    let mut byte: u8 = 0x80 | value;

     match arg_type {
        ArgType::Reference  => byte |= 0x01 << 4,
        ArgType::Variable   => byte |= 0x02 << 4,
        ArgType::SmallConst => byte |= 0x00 << 4,
        _                   => panic!("no possible 1OP")
    }

    vec![byte]
}

fn op_2( value: u8, arg_types: Vec<ArgType>) -> Vec<u8> {
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

fn encode_variable_arguments( arg_types: Vec<ArgType>) -> u8 {
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
