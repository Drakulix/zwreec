//! The `zfile` module contains ...
//! 

pub use super::zbytes::Bytes;
pub use super::ztext;

pub struct Zfile {
    pub data: Bytes,
    program_addr: u16
}




impl Zfile {

    pub fn new() -> Zfile {
        Zfile {
            data: Bytes{bytes: Vec::new()}, 
            program_addr: 0x800
        }
    }

    pub fn create_header(&mut self) {

        let alpha_addr: u16 = 0x40;
        let extension_addr: u16 = alpha_addr + 78;
        let global_addr: u16 = extension_addr as u16 + 4;
        let object_addr: u16 = global_addr + 480;
        let high_memory_addr: u16 = 0x308;
        let static_addr: u16 = 0x308;
        let dictionary_addr: u16 = 0x308;
        //let program_addr: u16 = 0x800;

        // version
        self.data.write_byte(8, 0x00);

        // flags
        self.data.write_byte(0, 0x01);

        // release version (0x02 und 0x03)
        self.data.write_u16(0, 0x02);

        // base of high memory (byte address) (0x04 and 0x05)
        self.data.write_u16(high_memory_addr, 0x04);

        // initial value of programm counter (0x06 and 0x07)
        self.data.write_u16(self.program_addr, 0x06);

        // location of dictionary (byte address) (0x08 and 0x09)
        self.data.write_u16(dictionary_addr, 0x08);

        // location of object table (byte address) (0x0a and 0x0b)
        self.data.write_u16(object_addr, 0x0a);

        // location of global variables table (byte address) (0x0c and 0x0d)
        self.data.write_u16(global_addr, 0x0c);

        // base of static memory (byte address) (0x0e and 0x0f)
        self.data.write_u16(static_addr, 0x0e);

        // alphabet address (bytes) - its 0x34 and 0x35, why not only 0x34?
        self.data.write_u16(alpha_addr, 0x34);

        // header extension table address (bytes) - its 0x36 and 0x37, why not only 0x36?
        self.data.write_u16(extension_addr, 0x36);

        // alphabet
        self.write_alphabet(alpha_addr as usize);

        // header extension table
        self.data.write_u16(3, extension_addr as usize); // Number of further words in table
        self.data.write_u16(0, extension_addr as usize + 1); // x-coordinate of mouse after a click
        self.data.write_u16(0, extension_addr as usize + 2); // y-coordinate of mouse after a click
        self.data.write_u16(0, extension_addr as usize + 3); // if != 0: unicode translation table address (optional)

        // global variables
        // ...

        // object tabelle name
        //data.write_u16(object_addr, 0x0c);
        //let tmp: [u8; 6] = [0x04, 0x50, 0xef, 0xa9, 0x19, 0x00];
        //data.write_bytes(&tmp, 770);
        //self.op_print("object", 770);
        self.write_object_name("object", 770);

        // dictionary
        let tmp: [u8; 4] = [0x00, 0x06, 0x00, 0x00];
        self.data.write_bytes(&tmp, dictionary_addr as usize);
    }

    // writes the alphabet to index
    fn write_alphabet(&mut self, index: usize) {
        // TODO: is it possible to do this with map?
        let mut alpha_tmp: [u8; 78] = [0; 78];
        for i in 0..ztext::ALPHA.len() {
            alpha_tmp[i] = ztext::ALPHA[i] as u8;
        }
        self.data.write_bytes(&alpha_tmp, index);
    }

    fn write_object_name(&mut self, name: &str, index: usize) {
        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        let length: u16 = ztext::encode(&mut text_bytes, name);

        // length ob object name
        self.data.write_byte(length as u8, index);

        self.data.write_bytes(&text_bytes.bytes, index + 1);
    }

    pub fn start(&mut self) {
        self.data.write_zero_until(self.program_addr as usize);
    }


    // ops

    pub fn op_print(&mut self, content: &str) {
        let index: usize = self.data.bytes.len() as usize;
        self.data.write_byte(0xb2, index);

        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        ztext::encode(&mut text_bytes, content);

        self.data.write_bytes(&text_bytes.bytes, index + 1);
    }

    pub fn op_quit(&mut self) {
        let index: usize = self.data.bytes.len() as usize;
        self.data.write_byte(0xb0 | 0x0a as u8, index);
    }
}