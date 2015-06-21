//! The `zcode` module contains a lot of useful functionality
//! to deal with all the zcode related stuff

pub mod zbytes;
pub mod zfile;
pub mod ztext;
pub mod op;

use self::zfile::Zfile;
use self::zfile::ZOP;

use std::error::Error;
use std::io::Write;


/// an example to show the current status of the z-code implementation
/// zcode playground function
pub fn temp_create_zcode_example<W: Write>(output: &mut W) {

    let mut zfile: Zfile = zfile::Zfile::new();

    zfile.start();
    let store_addr = zfile.object_addr + 2;
    // code for debugging purposes and can be changed as wanted
    zfile.emit(vec![
        ZOP::Routine{name: "Start".to_string(), count_variables: 1},
        ZOP::PrintOps{text: "Content at memory pointed by variable (expected 8729): \n".to_string()},
        ZOP::StoreU16{variable: 200, value: 0x2219},
        ZOP::StoreW{array_address: 2, index: 1, variable: 200},  // index at var:1 is 0
        ZOP::StoreU16{variable: 200, value: store_addr},
        ZOP::LoadWvar{array_address_var: 200, index: 1, variable: 190},
        ZOP::Newline,
        ZOP::PrintNumVar{variable: 190},
        ZOP::Newline,
        ZOP::PrintUnicodeVar{var: 190}, // should output ∙
        ZOP::Newline,
        ZOP::PrintUnicode{c: '∙' as u16},
        ZOP::Newline,
        ZOP::Quit,
        ]);
    zfile.end();

    match output.write_all(&(*zfile.data.bytes)) {
        Err(why) => {
            panic!("Could not write to output: {}", Error::description(&why));
        },
        Ok(_) => {
            info!("Wrote zcode to output");
        }
    };
}
