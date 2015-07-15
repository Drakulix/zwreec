//! The `zbyte` module contains code
//! to deal with opcodes and zcode.

/// A struct that holds an array of bytes and provides some convenience functions.
pub struct Bytes {
    /// The underlying data
    pub bytes: Vec<u8>
}

impl Bytes {
    /// Returns the length of the byte array.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Writes the byte (u8) to the index specified.
    ///
    /// If the vector isn't large enough it fills everything up to the index with zeros.
    pub fn write_byte(&mut self, byte: u8, index: usize) {
        while self.len() <= index {
            self.bytes.push(0);
        }

        self.bytes[index] = byte;
    }

    /// Appends a byte to the end of the data.
    pub fn append_byte(&mut self, byte: u8) {
        let index: usize = self.bytes.len();
        self.write_byte(byte, index);
    }

    /// Writes a u16 in two bytes with the correct byte-order for the Z-Machine at the specified
    /// index.
    pub fn write_u16(&mut self, value: u16, index: usize) {
        self.write_byte((value >> 8) as u8, index);
        self.write_byte((value & 0xff) as u8, index + 1);
    }

    /// Appends a u16 to the end of the data.
    pub fn append_u16(&mut self, value: u16) {
        let index: usize = self.bytes.len();
        self.write_u16(value, index);
    }

    /// Writes multiple bytes at the specified index.
    pub fn write_bytes(&mut self, bytes: &[u8], to_index: usize) {
        for i in 0..bytes.len() {
            self.write_byte(bytes[i], to_index+i);
        }
    }

    /// Appends an array of bytes at the end of the data.
    pub fn append_bytes(&mut self, bytes: &[u8]) {
        let index: usize = self.bytes.len();
        self.write_bytes(bytes, index);
    }

    /// Fills everything with zeros until but not including the index.
    ///
    /// `=> [index-1] == 0; [index] == nil;`
    pub fn write_zero_until(&mut self, index: usize) {
        while self.len() < index {
            self.bytes.push(0);
        }
    }

    /// Prints the underlying byte array
    pub fn print(&self) {
        debug!("bytes: {:?}", self.bytes);
    }
}
