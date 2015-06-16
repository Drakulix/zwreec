pub use super::zfile::ArgType;
/// decrements the value of the variable

pub fn op_dec(variable: u8) -> Vec<u8> {
    let mut bytes = op_1(0x06, ArgType::Reference);
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
