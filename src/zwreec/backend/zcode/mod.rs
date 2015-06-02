//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod zbytes;
pub mod zfile;
pub mod ztext;

use utils::file;
use self::zfile::Zfile;


/// an example to show the current status of the z-code implementation
/// zcode playground function
pub fn temp_create_zcode_example() {

    let mut zfile: Zfile = zfile::Zfile::new();

    zfile.start();
    zfile.op_call_1n("Start");

    zfile.routine("Start", 0);
    zfile.op_print("start passage");
    zfile.op_newline();


    zfile.op_quit();
    zfile.end();

    file::save_bytes_to_file("helloworld.z8", &(*zfile.data.bytes));
}
