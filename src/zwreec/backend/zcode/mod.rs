//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod zbytes;
pub mod zfile;
pub mod ztext;

use utils::file;
use self::zfile::Zfile;


/// an example to show the current status of the z-code implementation
pub fn temp_create_zcode_example() {

    let mut zfile: Zfile = zfile::Zfile::new();
    zfile.start();
    zfile.op_call_1n("main");
    zfile.op_quit();

    zfile.routine("main", 0);
    zfile.op_print("Z-char 6789abcdef0123456789abcdef");
    zfile.op_newline();
    zfile.op_print("current   --------------------------");
    zfile.op_newline();
    zfile.op_print("A0      abcdefghijklmnopqrstuvwxyz");
    zfile.op_newline();
    zfile.op_print("A1      ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    zfile.op_newline();
    zfile.op_print("A2       0123456789.,!?_#'\"/\\-:()");
    zfile.op_newline();
    zfile.op_print("          --------------------------");
    zfile.op_newline();

    zfile.op_quit();
    zfile.end();

    file::save_bytes_to_file("helloworld.z8", &(*zfile.data.bytes));
}

pub fn temp_hello() -> String {
    "hello from zcode".to_string()
}
