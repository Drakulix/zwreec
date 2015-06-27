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
        ZOP::Add{operand1: Operand::new_var(1), operand2: Operand::new_var(1), save_variable: Variable::new(1)},
        ZOP::Call2S{jump_to_label: "itoa".to_string(), arg: Operand::new_large_const(17), result: Variable::new(1)},
        ZOP::PrintUnicodeStr{address: Operand::new_var(1)},
        ZOP::PrintOps{text: "it works".to_string()},
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
