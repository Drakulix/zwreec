//! The `ztext` module contains encoding functions to encode text in z-ascii characters.
//! 

use super::zbytes::Bytes;

pub static ALPHA: [char; 78] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',

    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',

    '\0', '^', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.',
    ',', '!', '?', '_', '#', '\'','"', '/', '\\','-', ':', '(', ')'];


/// encodes an string to z-characters
/// and returns the length of the used bytes
/// TODO: Only works with lower case letters
pub fn encode(data: &mut Bytes, content: &str) -> u16 {
    let string_bytes = content.to_string().into_bytes();

    let mut two_bytes: u16 = 0;
    let len = string_bytes.len();
    for i in 0..len {
        let letter = string_bytes[i];
        let zasci_id = pos_in_alpha(letter as u8) % 26 + 6;
        println!("zasci_id: {:?}", zasci_id);

        two_bytes |= (zasci_id as u16) << shift(i as u8);

        if i % 3 == 2 {
            data.write_u16(two_bytes, pos_to_index(i));
            two_bytes = 0;
        }

        // end of string
        if i == string_bytes.len() -1 {
            if i % 3 != 2 {
                for j in (i % 3) + 1..3 {
                    two_bytes |= (0x05 as u16) << shift(j as u8);
                }

                data.write_u16(two_bytes, pos_to_index(i));
            }

            // end bit is written to the first bit of the next to last byte
            let end_index: usize = data.bytes.len() - 2 as usize;
            let mut end_byte: u8 = data.bytes[end_index];

            end_byte |= 0x80;
            data.write_byte(end_byte, end_index as usize);
        }
    }

    /// Shifts the data for the corresponding position in the 2-byte array
    fn shift(position: u8) -> u8 {
        10 - (position % 3) * 5
    }

    /// Returns the location of the character of the specified index in the zcode character array
    fn pos_in_alpha(letter: u8) -> i8 {
        for i in 0..ALPHA.len() {
            if ALPHA[i] as u8 == letter {
                return i as i8
            }
        }

        return -1
    }

    /// position in the vector from the position of an character in the string
    fn pos_to_index(position: usize) -> usize {
        2 * (position / 3)
    }

    data.bytes.len() as u16
}
