//! The `zbyte` module contains ...
//! to deal with opcodes and zcode

pub struct Bytes {
    pub bytes: Vec<u8>
}

impl Bytes {
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn write_byte(&mut self, byte: u8, index: usize) {
        /*if index >= self.len() {
            for _ in 0..(index - self.len() + 1) {
                self.bytes.push(0);
            }
        }*/
        while self.len() <= index {
            self.bytes.push(0);
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

    // fills everything with zeros until the index
    // => [index-1] == 0; [index] == nil;
    pub fn write_zero_until(&mut self, index: usize) {
        while self.len() < index {
            self.bytes.push(0);
        }
    }

    pub fn print(&self) {
        println!("bytes: {:?}", self.bytes);

    }
}
