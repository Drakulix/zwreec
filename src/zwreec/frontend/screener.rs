//! Sanitizes the input stream.

use std::error::Error;
use std::io::{BufReader,Cursor,Read};

/// Checks for and removes a UTF-8 Byte Order Mark (BOM) from the input stream.
pub fn handle_bom_encoding<'a, R: Read>(input: &'a mut R) -> Cursor<Vec<u8>> {
    let mut reader = BufReader::new(input);
    let mut content = String::new();
    match reader.read_to_string(&mut content) {
        Err(why) => error!("Couldn't read {}", Error::description(&why)),
        Ok(_) => (),
    };

    let mut bytes: Vec<u8> = content.bytes().collect();
    if bytes.len() < 5 {
        error!("The file is too short for a valid twee file");
    }
    let has_bom = if &bytes[0..3] == [0xef, 0xbb, 0xbf] {
        true
    } else {
        false
    };
    if has_bom {
        debug!("File has UTF-8 Byte Order Mark (BOM): Removing the first three bytes from the file");
        bytes.remove(0);
        bytes.remove(0);
        bytes.remove(0);
    }

    let cursor: Cursor<Vec<u8>> = Cursor::new(bytes);

    cursor
}
