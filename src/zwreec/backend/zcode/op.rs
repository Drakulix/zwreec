//! The `op` module contains code
//! to deal with opcodes

pub fn quit() -> Vec<u8> {
    op_0(0x0a)
}

/// op-codes with 0 operators
fn op_0(value: u8) -> Vec<u8> {
    let byte = value | 0xb0;

    vec![byte]
}
