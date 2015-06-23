//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod zbytes;
pub mod zfile;
pub mod ztext;
pub mod op;

use self::zfile::{Zfile, Operand, Variable, ZOP};

use std::error::Error;
use std::io::Write;


/// an example to show the current status of the z-code implementation
/// zcode playground function
pub fn temp_create_zcode_example<W: Write>(output: &mut W) {

    let mut zfile: Zfile = zfile::Zfile::new();

    zfile.start();
    zfile.emit(vec![
        ZOP::Routine{name: "Start".to_string(), count_variables: 3},
        ZOP::StoreVariable{variable: Variable::new(200), value: Operand::new_large_const(0x0002)},
        ZOP::StoreVariable{variable: Variable::new(201), value: Operand::new_large_const(0x0004)},
        ZOP::StoreVariable{variable: Variable::new(202), value: Operand::new_large_const(0x1000)},
        ZOP::Mul{operand1: Operand::new_var(200), operand2:Operand::new_var(201) , save_variable: Variable::new(202)},
        ZOP::Newline,
        ZOP::PrintNumVar{variable: Variable::new(202)},
        ZOP::Newline,
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
