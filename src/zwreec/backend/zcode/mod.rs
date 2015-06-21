//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod zbytes;
pub mod zfile;
pub mod ztext;
pub mod op;

use self::zfile::{Zfile, Operand, Variable};

use std::error::Error;
use std::io::Write;


/// an example to show the current status of the z-code implementation
/// zcode playground function
pub fn temp_create_zcode_example<W: Write>(output: &mut W) {

    let mut zfile: Zfile = zfile::Zfile::new();

    zfile.start();
    // zfile.op_call_1n("Start");
    zfile.routine("Start", 0);
    zfile.gen_print_ops("Address of var 200: \n");
    op::op_store_var(&Variable::new(200), &Operand::new_large_const(0x103));
    // zfile.op_print_paddr(200);
    op::op_newline();
    op::op_newline();

    zfile.op_quit();
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
