//! A module for using `zcode`s.
//!
//! The `zcode` module contains a lot of useful functionality
//! to deal with opcodes and zcode

use file;

pub struct Bytes {
    bytes: Vec<u8>
}

impl Bytes {
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn write_byte(&mut self, byte: u8, index: usize) {
        if index >= self.len() {
            for _ in 0..(index - self.len() + 1) {
                self.bytes.push(0);
            }
        }
        
        self.bytes[index] = byte;
    }

    pub fn temp_print(&self) {
        println!("bytes: {:?}", self.bytes);
    }
}

fn header(data: &mut Bytes) {

    // version
    data.write_byte(8, 0x00);

    // flags
    data.write_byte(0, 0x01);

    // alphabet address
    data.write_byte(64, 0x34);

    // header extension table address
    data.write_byte(0, 0x36);    

    // alphabet


}

pub fn temp_create_hello_world_zcode() {
    
    let mut data: Bytes = Bytes{bytes: Vec::new()};

    header(&mut data);
    data.temp_print();


    file::save_bytes_to_file("helloworld.z8", &*data.bytes);
}

pub fn temp_hello() -> String {
    "hello from zcode".to_string()
}

#[test]
fn it_works() {

}
