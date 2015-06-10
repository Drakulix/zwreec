//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod zbytes;
pub mod zfile;
pub mod ztext;

use std::error::Error;
use std::io::Write;
use self::zfile::Zfile;


/// an example to show the current status of the z-code implementation
/// zcode playground function
pub fn temp_create_zcode_example<W: Write>(output: &mut W) {

    let mut zfile: Zfile = zfile::Zfile::new();

    zfile.start();
    zfile.op_call_1n("Start");

    zfile.routine("Start", 0);
    zfile.op_print("start passage");
    zfile.op_newline();


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
