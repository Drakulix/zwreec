use std::io::{Cursor, Read};
use std::error::Error;
use utils::extensions::{PeekingExt, FilteringScanExt};

// TODO: do not remove if in passage name, monospace etc.

struct ScanState {
    comment: bool,
    skip_next: bool,
}

pub fn screen<R: Read>(input: &mut R) -> Cursor<Vec<u8>> {

    let mut content = String::new();
    match input.read_to_string(&mut content) {
        Err(why) => panic!("could not read from input: {}", Error::description(&why)),
        Ok(_) => debug!("read input to buffer"),
    };

    Cursor::new(content.chars().peeking().scan_filter(
        ScanState {
            comment: false,
            skip_next: false,
        },
        |state, elem| {
            match (state.comment, state.skip_next, elem) {
                (_, true, _) => { //skipping
                    state.skip_next = false;
                    None
                },
                (false, _, ('/', Some('%'))) => { //comment_start
                    state.comment = true;
                    state.skip_next = true;
                    None
                },
                (true, _, ('%', Some('/'))) => { //comment_end
                    state.comment = false;
                    state.skip_next = true;
                    None
                },
                (true, _, _) => None, //comment
                (false, _, (x, _)) => Some(x as u8), //char
            }
        }
    ).collect())
}
