//! The `ztext` module contains encoding functions to encode text in z-ascii characters.
//! 

use super::zbytes::Bytes;

pub static ALPHABET: [char; 78] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',

    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',

    '\0', '^', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.',
    ',', '!', '?', '_', '#', '\'','"', '/', '\\','-', ':', '(', ')'];


/// encodes an string to z-characters
/// and returns the length of the used bytes
/// TODO: Only works with lower case letters
///
/// # Examples
///
/// ```ignore
/// let data = Bytes{bytes: Vec::new()};
/// let byteLength = data.encode("hello");
/// ```
pub fn encode(data: &mut Bytes, content: &str) -> u16 {
    let string_bytes = content.to_string().into_bytes();
    let mut temp: Vec<i8> = Vec::new();
    for i in string_bytes{
        let t_index = pos_in_alpha(i as u8);
        if t_index <27 {
            temp.push(t_index % 26 + 6);
        } else {
            temp.push(0x04 as i8);
            temp.push(t_index % 26 + 6);
        } 
    }

    let mut two_bytes: u16 = 0;
    let len = temp.len();
    for i in 0..len {
        let zasci_id =temp[i];

        //two_bytes |= (zasci_id as u16) << shift(i as u8);
        two_bytes |= shift(zasci_id as u16, i as u8);

        if i % 3 == 2 {
            data.write_u16(two_bytes, pos_to_index(i));
            two_bytes = 0;
        }

        // end of string
        if i == len -1 {
            if i % 3 != 2 {
                for j in (i % 3) + 1..3 {
                    //two_bytes |= (0x05 as u16) << shift(j as u8);
                    two_bytes |= shift(0x05 as u16, j as u8);
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

    data.bytes.len() as u16
}

/// shifts the z-char in a 2 bytes-array to the right position
/// shift_length has 3 possibilities: 10, 5, 0
/// an z-char use 5 bytes
/// look into two bytes:
/// ____byte one____ ____byte two____
/// 1 2 3 4 5 6  7 8 1 2 3  4 5 6 7 8
///   ^          ^          1
///   1 2 3 4 5  1 2 3 4 5  1 2 3 4 5
///   1. zchar   2. zchar   3. zchar 
///   10         5          0
fn shift(zchar: u16, position: u8) -> u16 {
    let shift_length = 10 - (position % 3) * 5;
    zchar << shift_length
}

/// Returns the location of the character of the specified index in the zcode character array
///  
/// # Examples
///
/// ```ignore
/// assert_eq!(pos_in_alpha('c'), 2);
/// ```
fn pos_in_alpha(letter: u8) -> i8 {
    for i in 0..ALPHABET.len() {
        if ALPHABET[i] as u8 == letter {
            return i as i8
        }
    }

    return -1
}

/// position in the vector from the position of an character in the string
/// every 3 zchars encode() writes 2 bytes
/// for example "helloworld" 
/// "h": nothing
/// "e": nothing
/// "l": write 2 bytes to position 0   (2 * 2/3))
/// "l": nothing
/// "o": nothing
/// "w": write 2 bytes to position 2   (2 * 5/3))
/// ...
///
/// # Examples
///
/// ```ignore
/// assert_eq!(pos_to_index(5), 2);
/// ```
fn pos_to_index(position: usize) -> usize {
    2 * (position / 3)
}
