//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod zbytes;
pub mod zfile;
pub mod ztext;

use file;
use self::zfile::Zfile;


pub fn temp_create_hello_world_zcode() {
    
    let mut zfile: Zfile = zfile::Zfile::new();

    zfile.create_header();

    // endless loop hello world program
    zfile.start();
    zfile.op_call_1n("main");
    zfile.op_quit();
    zfile.routine("main", 0);
    zfile.label("repeat");
    zfile.op_print("hellofrommain");
    let local_var_id = 0x03;
    zfile.op_read_char(local_var_id);
    zfile.op_je(local_var_id, '1' as u8, "repeat");
    zfile.op_quit();
    zfile.end();


    file::save_bytes_to_file("helloworld.z8", &(*zfile.data.bytes));
}

pub fn temp_hello() -> String {
    "hello from zcode".to_string()
}

#[test]
fn it_works() {

}
