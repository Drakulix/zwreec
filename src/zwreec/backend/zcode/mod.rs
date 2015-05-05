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
    zfile.op_print("main");

    zfile.label("loop");
    let local_var_id = 1;
    zfile.op_read_char(local_var_id);
    zfile.op_je(local_var_id, '1' as u8, "one");
    zfile.op_je(local_var_id, '2' as u8, "two");
    zfile.op_je(local_var_id, '3' as u8, "end");
    zfile.op_jump("loop");

    zfile.label("one");
    zfile.op_print("one");
    zfile.op_jump("loop");

    zfile.label("two");
    zfile.op_print("two");
    zfile.op_jump("loop");

    zfile.label("end");
    zfile.op_quit();
    zfile.end();

    file::save_bytes_to_file("helloworld.z8", &(*zfile.data.bytes));
}

pub fn temp_hello() -> String {
    "hello from zcode".to_string()
}
