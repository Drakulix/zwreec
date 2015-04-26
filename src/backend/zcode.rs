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

    pub fn write_u16(&mut self, value: u16, index: usize) {
        self.write_byte((value >> 8) as u8, index);
        self.write_byte((value & 0xff) as u8, index + 1);
    }

    pub fn write_bytes(&mut self, bytes: &[u8], to_index: usize) {
        for i in 0..bytes.len() {
            self.write_byte(bytes[i], to_index+i);
        }
    }

    /*pub fn write_object_name(&mut self, name: &str, index: usize) {
        // length
        self.write_byte(1, index);
    }*/

    pub fn temp_print(&self) {
        println!("bytes: {:?}", self.bytes);

    }

    pub fn op_quit(&mut self) {
        let index: usize = self.bytes.len() as usize;
        self.write_byte(0xb0 | 0x0a as u8, index);
    }

    pub fn op_print(&mut self, content: &str, index: usize) {
        self.write_byte(0xb2, index);

        let string_bytes = content.to_string().into_bytes();

        let mut two_bytes: u16 = 0;
        let len = string_bytes.len();
        for i in 0..len {
            let letter = string_bytes[i];
            let pos = pos_in_alpha(letter as u8) + 6;

            two_bytes = two_bytes | (pos as u16) << shift(i as u8);

            if i % 3 == 2 {
                self.write_u16(two_bytes, index + ((i / 3) as usize) * 2 + 1);
                two_bytes = 0;
            }

            // end of string
            if i == string_bytes.len() -1 {
                if i % 3 != 2 {
                    for j in (i % 3) + 1..3 {
                        println!("2bytes end: {} - {}", j, i % 3);
                        two_bytes = two_bytes | (0x05 as u16) << shift(j as u8);
                        println!("2bytes end");
                    }

                    self.write_u16(two_bytes, index + ((i / 3) as usize) * 2 + 1);
                }
                
                let count_bytes: u16 = (((len as u16 - 1) / 3) + 1) * 2;

                // end bit is written to the first bit in the pre last byte
                let pre_last_byte_index: usize = index + count_bytes as usize-1;
                let mut pre_last_byte: u8 = self.bytes[pre_last_byte_index];
                pre_last_byte = pre_last_byte | 0x80;
                self.write_byte(pre_last_byte, pre_last_byte_index as usize);
            }
        }

        fn shift(position: u8) -> u8 {
            10 - (position % 3) * 5
        }

        fn pos_in_alpha(letter: u8) -> i8 {
            for i in 0..ALPHA.len() {
                if ALPHA[i] as u8 == letter {
                    return i as i8
                }
            }

            return -1
        }
    }
}


static ALPHA: [char; 78] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',

    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',

    '\0', '^', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.',
    ',', '!', '?', '_', '#', '\'','"', '/', '\\','-', ':', '(', ')'];


pub fn temp_create_hello_world_zcode() {
    
    let mut data: Bytes = Bytes{bytes: Vec::new()};

    // header
    // setup addresses (perhaps in the feature more dynamic?)
    let alpha_addr: u16 = 0x40;
    let extension_addr: u16 = alpha_addr + 78;
    let global_addr: u16 = extension_addr as u16 + 4;
    let object_addr: u16 = global_addr + 480;
    let high_memory_addr: u16 = 0x308;
    let static_addr: u16 = 0x308;
    let dictionary_addr: u16 = 0x308;
    let program_addr: u16 = 0x800;


    // version
    data.write_byte(8, 0x00);

    // flags
    data.write_byte(0, 0x01);

    // release version (0x02 und 0x03)
    data.write_u16(0, 0x02);

    // base of high memory (byte address) (0x04 and 0x05)
    data.write_u16(high_memory_addr, 0x04);

    // initial value of programm counter (0x06 and 0x07)
    data.write_u16(program_addr, 0x06);

    // location of dictionary (byte address) (0x08 and 0x09)
    data.write_u16(dictionary_addr, 0x08);

    // location of object table (byte address) (0x0a and 0x0b)
    data.write_u16(object_addr, 0x0a);

    // location of global variables table (byte address) (0x0c and 0x0d)
    data.write_u16(global_addr, 0x0c);

    // base of static memory (byte address) (0x0e and 0x0f)
    data.write_u16(static_addr, 0x0e);

    // alphabet address (bytes) - its 0x34 and 0x35, why not only 0x34?
    data.write_u16(alpha_addr, 0x34);

    // header extension table address (bytes) - its 0x36 and 0x37, why not only 0x36?
    data.write_u16(extension_addr, 0x36);

    // alphabet
    // TODO: is it possible to do this with map?
    let mut alpha_tmp: [u8; 78] = [0; 78];
    for i in 0..ALPHA.len() {
        alpha_tmp[i] = ALPHA[i] as u8;
    }
    data.write_bytes(&alpha_tmp, alpha_addr as usize);

    // header extension table
    data.write_u16(3, extension_addr as usize); // Number of further words in table
    data.write_u16(0, extension_addr as usize + 1); // x-coordinate of mouse after a click
    data.write_u16(0, extension_addr as usize + 2); // y-coordinate of mouse after a click
    data.write_u16(0, extension_addr as usize + 3); // if != 0: unicode translation table address (optional)

    // global variables
    // ...

    // object tabelle name
    //data.write_u16(object_addr, 0x0c);
    //let tmp: [u8; 6] = [0x04, 0x50, 0xef, 0xa9, 0x19, 0x00];
    //data.write_bytes(&tmp, 770);
    data.op_print("object", 770);

    // dictionary
    let tmp: [u8; 4] = [0x00, 0x06, 0x00, 0x00];
    data.write_bytes(&tmp, dictionary_addr as usize);


    // hello world program
    data.op_print("worlhsfgsdfgsdfg", program_addr as usize);


    data.op_quit();


    // bytes at the end
    //let tmp: [u8; 2] = [0x00, 0x00];
    //data.write_bytes(&tmp, 2054);


    file::save_bytes_to_file("helloworld.z8", &*data.bytes);
}

pub fn temp_hello() -> String {
    "hello from zcode".to_string()
}

#[test]
fn it_works() {

}
