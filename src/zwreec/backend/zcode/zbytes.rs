//! The `zbyte` module contains code
//! to deal with opcodes and zcode

pub struct Bytes {
    pub bytes: Vec<u8>
}

impl Bytes {
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// writes on byte (in u8) to the index
    /// if vector isn't large enough:
    ///     fills everything until index with zero
    pub fn write_byte(&mut self, byte: u8, index: usize) {
        while self.len() <= index {
            self.bytes.push(0);
        }
        
        self.bytes[index] = byte;
    }

    /// appends a byte to the end of self.bytes
    pub fn append_byte(&mut self, byte: u8) {
        let index: usize = self.bytes.len();
        self.write_byte(byte, index);
    }

    /// writes u16 in two bytes at the specified index
    pub fn write_u16(&mut self, value: u16, index: usize) {
        self.write_byte((value >> 8) as u8, index);
        self.write_byte((value & 0xff) as u8, index + 1);
    }

    /// appends a u16 to the end of self.bytes
    pub fn append_u16(&mut self, value: u16) {
        let index: usize = self.bytes.len();
        self.write_u16(value, index);
    }

    /// writes multiple bytes at the specified index
    pub fn write_bytes(&mut self, bytes: &[u8], to_index: usize) {
        for i in 0..bytes.len() {
            self.write_byte(bytes[i], to_index+i);
        }
    }

    /// fills everything with zeros until but not including the index
    /// => [index-1] == 0; [index] == nil;
    pub fn write_zero_until(&mut self, index: usize) {
        while self.len() < index {
            self.bytes.push(0);
        }
    }

    pub fn print(&self) {
        debug!("bytes: {:?}", self.bytes);

    }
}
