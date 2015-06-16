pub use super::zfile::ArgType;













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
pub fn op_sub(variable1: u8, sub_const: u16, variable2: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::LargeConst];
    let mut bytes = op_2(0x15, args);
    bytes.push(variable1);
	write_u16(sub_const, &mut bytes);
    bytes.push(variable2);
    bytes
}


/// addition
/// variable2 = variable1 + sub_const
pub fn op_add(variable1: u8, add_const: u16, variable2: u8) -> Vec<u8> {
    let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::LargeConst];
    let mut bytes = op_2(0x14, args);
    bytes.push(variable1);
	write_u16(add_const, &mut bytes);
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
