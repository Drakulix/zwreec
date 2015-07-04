//! The `screener` module checks the inputstream before the lexer will get it

use std::error::Error;
use std::io::{BufReader,Cursor,Read};

/// checks the input if there is an bom, if true it will delete it
pub fn handle_bom_encoding<'a, R: Read>(input: &'a mut R) -> Cursor<Vec<u8>> {
    let mut reader = BufReader::new(input);
    let mut content = String::new();
    match reader.read_to_string(&mut content) {
        Err(why) => error!("Couldn't read {}", Error::description(&why)),
        Ok(_) => (),
    };

    let mut v: Vec<u8> = content.bytes().collect();
    if v.len() < 5 {
        error!("The file is to short for a valid twee File");
    }
    let has_bom = if &v[0..3] == [0xef, 0xbb, 0xbf] {
        true
    } else {
        false
    };
    if has_bom {
        debug!("File has Byte order mark");
        v.remove(0);
        v.remove(0);
        v.remove(0);
    }

    let cursor: Cursor<Vec<u8>> = Cursor::new(v);

    cursor
}