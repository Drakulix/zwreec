//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod op;
pub mod zbytes;
pub mod zfile;
pub mod ztext;


use std::error::Error;
use std::io::Write;

use self::zfile::{Zfile, Operand, Variable, ZOP, Type};


/// an example to show the current status of the z-code implementation
/// zcode playground function
pub fn temp_create_zcode_example<W: Write>(output: &mut W) {

    let mut zfile: Zfile = zfile::Zfile::new();

    zfile.start();
    zfile.emit(vec![
        ZOP::Routine{name: "Start".to_string(), count_variables: 14},
        ZOP::StoreVariable{variable: Variable::new(1), value: Operand::new_large_const(1337)},
        ZOP::Call2S{jump_to_label: "itoa".to_string(), arg: Operand::new_large_const(1337), result: Variable::new(2)},
        ZOP::SetVarType{variable: Variable::new(1), vartype: Type::Integer},
        ZOP::SetVarType{variable: Variable::new(2), vartype: Type::String},
        ZOP::AddTypes{operand1: Operand::new_var(1), operand2: Operand::new_var(2), tmp1: Variable::new(3), tmp2: Variable::new(4), save_variable: Variable::new(1)},
        ZOP::AddTypes{operand1: Operand::new_var(1), operand2: Operand::new_var(2), tmp1: Variable::new(3), tmp2: Variable::new(4), save_variable: Variable::new(1)},
        ZOP::PrintUnicodeStr{address: Operand::new_var(1)},
        ZOP::Newline,
        ZOP::PrintVar{variable: Variable::new(1)},
        ZOP::Quit,
        ]);
    zfile.end();

    match output.write_all(&(*zfile.data.bytes)) {
        Err(why) => {
            panic!("Could not write to output: {}", Error::description(&why));
        },
        Ok(_) => {
            info!("Wrote zcode to output");
        }
    };
}
