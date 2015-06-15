use std::io::{Cursor, Read};
use std::error::Error;

pub fn screen<R: Read>(input: &mut R) -> Cursor<Vec<u8>> {

    let mut content = String::new();
    match input.read_to_string(&mut content) {
        Err(why) => panic!("could not read from input: {}", Error::description(&why)),
        Ok(_) => debug!("read input to buffer"),
    };

    Cursor::new(content.bytes().collect())
}
