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

    zfile.op_print("link1");
    zfile.op_call_2n_with_address("system_add_link", "link1");

    zfile.op_print("link2");
    zfile.op_call_2n_with_address("system_add_link", "link2");

    zfile.op_newline();
    zfile.op_call_1n("system_check_links");

    zfile.routine("link1", 1);
    zfile.op_print("passage 1");
    zfile.op_newline();
    zfile.op_call_1n("Start");

    zfile.routine("link2", 1);
    zfile.op_print("passage 2");
    zfile.op_newline();
    zfile.op_call_1n("Start");

    //zfile.op_loadw(0, 0x01, 0x02);

    //zfile.op_storew(1234, 16, 0x01);
    //zfile.op_loadw(1234, 16, 0x01);

    //zfile.op_ret();
    //zfile.op_sub(0x01, 1234, 0x01);
    //zfile.op_call_2n_with_address("test", "link1");
    //zfile.routine("test", 1);
    //zfile.routine("link1", 0);
    //zfile.op_call_1n_var(0x01);
    //zfile.routine("test", 1);
    zfile.op_quit();


    /*zfile.routine("main", 1);
    zfile.op_print("main");
    zfile.op_newline();


    zfile.op_print("passage1");    
    zfile.op_call_2n_with_address("system_add_link", "passage1");


    zfile.routine("passage1", 1);

    zfile.op_print_num_var(0x01);
    zfile.op_print("-");
    zfile.op_print_num_var(0x30);
    zfile.op_newline();
    zfile.op_push_u16(1234);
    zfile.op_pull(0x20);

    zfile.op_print("Var 0x20: ");
    zfile.op_print_num_var(0x20);
    zfile.op_newline();
    zfile.op_inc(0x01);
    zfile.op_print("Var 0x01: ");
    zfile.op_print_num_var(0x01);
    zfile.op_newline();
    zfile.op_inc(0x01);
    zfile.op_print("Var 0x01: ");
    zfile.op_print_num_var(0x01);
    zfile.op_newline();

    zfile.label("loop");
    let local_var_id = 1;
    zfile.op_read_char(local_var_id);
    zfile.op_print_num_var(local_var_id);
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
    
    zfile.op_quit();*/
    zfile.end();

    file::save_bytes_to_file("helloworld.z8", &(*zfile.data.bytes));
}
