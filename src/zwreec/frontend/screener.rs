use std::io::{Cursor, Read};
use std::error::Error;
use utils::extensions::{QueuedScanExtension, QueuedScan};

// TODO: do not remove if in passage name, monospace etc.

struct ScanState {
    comment: bool,
    suspect_start: bool,
    suspect_end: bool,
}

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

    Cursor::new(content.chars().queued_scan(
        ScanState {
            comment: false,
            suspect_start: false,
            suspect_end: false
        },
        |ref mut state, maybe_char, ref mut queue| {
            match maybe_char {
                Some(c) => {
                    if !state.comment && !state.suspect_start && c == '/' {
                        state.suspect_start = true;
            			return true;
            		}

            		if state.suspect_start {
            			if c == '%' {
                            state.comment = true;
                            state.suspect_start = false;
            			} else {
                            state.suspect_start = false;
            				queue.push_back('/');
                            queue.push_back(c);
            			}
                        return true;
            		}

            		if c == '%' && state.comment {
                        state.suspect_end = true;
                        return true;
            		}

            		if state.suspect_end {
            			if c == '/' {
                            state.comment = false;
                            state.suspect_end = false;
            			} else {
                            state.suspect_end = false;
            			}
                        return true;
            		}

            		if !state.comment {
                        queue.push_back(c);
            		}

                    true
                },
                None => false
            }
        }
    ).map(|x: char| {x as u8}).collect())
}
