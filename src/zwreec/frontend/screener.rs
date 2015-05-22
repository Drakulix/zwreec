use std::io::{Cursor, Read};
use std::error::Error;

// TODO: do not remove if in passage name, monospace etc.

pub fn screen<R: Read>(input: &mut R) -> Cursor<Vec<u8>> {

    let mut content = String::new();
    match input.read_to_string(&mut content) {
        Err(why) => panic!("could not read from input: {}", Error::description(&why)),
        Ok(_) => debug!("read input to buffer"),
    };

    let mut comment = false;
	let mut suspect_start = false;
	let mut suspect_end = false;
	let mut processed = String::new();

    //I really would have loved to use the chars iterator and filter or scan to perform this operation.
    //Sadly the comment blocks pervent this, as they depend on more then one chars to start
    //"fold" would have been possible, but is not more efficient then this old-school variant.
	for c in content.chars() {
		if !comment && !suspect_start && c == '/' {
			suspect_start = true;
			continue;
		}

		if suspect_start {
			if c == '%' {
				comment = true;
				suspect_start = false;
			} else {
				suspect_start = false;
				processed.push('/');
				processed.push(c);
			}

			continue;
		}

		if c == '%' && comment {
			suspect_end = true;
			continue;
		}

		if suspect_end {
			if c == '/' {
				comment = false;
				suspect_end = false;
			} else {
				suspect_end = false;
			}
			continue;
		}

		if !comment {
			processed.push(c);
		}
	}

    Cursor::new(processed.into_bytes())
}
