//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod zbytes;
pub mod zfile;
pub mod ztext;

use file;
pub use self::zbytes::Bytes;
pub use self::zfile::Zfile;
//pub use Bytes;



// NEXT
// function die berechnet aus wieviele bytes ein string besteht
// op_print auslagern in eine function die den string als vector zurückgibt
// und damit allgeim genutzt werden kann


pub fn temp_create_hello_world_zcode() {
    
    //let mut data: zbytes::Bytes = zbytes::Bytes{bytes: Vec::new()};
    let mut zfile: Zfile = zfile::Zfile::new();

    zfile.create_header();

    // hello world program
    zfile.start();
    zfile.op_call_1n("main");
    zfile.op_quit();
    zfile.routine("main", 0);
    zfile.op_print("hellofrommain");
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
