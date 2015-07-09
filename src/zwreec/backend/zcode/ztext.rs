//! The `ztext` module contains encoding functions to encode text in z-ascii characters.
//! 

use super::zbytes::Bytes;

/// the ascii-alphabet with the 3 parts:
/// lower-case, upper-case and numbers / special characters
pub static ALPHABET: [char; 78] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',

    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',

    ' ', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.',
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
pub fn encode(data: &mut Bytes, content: &str, unicode_table: &Vec<u16>) -> u16 {
    let zchars: Vec<u8> = string_to_zchar(content, unicode_table);

    let mut two_bytes: u16 = 0;
    let len = zchars.len();
    for i in 0..len {
        let zasci_id =zchars[i];

        two_bytes |= shift(zasci_id as u16, i);

        if i % 3 == 2 {
            data.write_u16(two_bytes, pos_to_index(i));
            two_bytes = 0;
        }

        // end of string
        if i == len -1 {
            if i % 3 != 2 {
                for j in (i % 3) + 1..3 {
                    two_bytes |= shift(0x05 as u16, j);
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

/// reads the content and converts it to a zasci vector
fn string_to_zchar(content: &str, unicode_table: &Vec<u16>) -> Vec<u8> {
    //let string_bytes = content.to_string().into_bytes();
    let mut zchars: Vec<u8> = Vec::new();

    for character in content.chars() {

        let mut byte: u8 = character as u8;
        let alpha_index = pos_in_alpha(byte as u8);
        if character as u16 <= 126 && alpha_index != -1 {

            if byte == 0x0A {
                // newline
                zchars.push(0x05);
                zchars.push(7);
            } else if byte == 0x20 {
                // space
                zchars.push(0x00);
            } else {
                if alpha_index > 51 {
                    // in A2
                    zchars.push(0x05);  
                    zchars.push(alpha_index as u8 % 26 + 6);
                } else if alpha_index < 26 {
                    // in A0
                    zchars.push(alpha_index as u8 % 26 + 6);
                } else {
                    // in A1
                    zchars.push(0x04);
                    zchars.push(alpha_index as u8 % 26 + 6);
                } 
            }
        } else {
            // not in alphabet or unicode

            // to change alphabet
            zchars.push(0x05);

            // for special char (10 bit z-ascii)
            zchars.push(0x06);

            //let mut byte: u8;
            if character as u16 <= 126 {
                // not in alphabet, but still ascii
                byte = character as u8;
            } else {
                // unicode
                let unicode_index = pos_in_unicode(character as u16, unicode_table);
                byte = unicode_index as u8 + 155;
            }

            zchars.push(byte >> 5);
            zchars.push(byte & 0x1f);
        }
    }
    zchars
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
fn shift(zchar: u16, position: usize) -> u16 {
    let shift_length = 10 - (position % 3) as u16 * 5;
    zchar << shift_length
}

/// Returns the location of the character of the specified index in the zcode character array
///  
/// # Examples
///
fn pos_in_alpha(letter: u8) -> i8 {
    for i in 0..ALPHABET.len() {
        if ALPHABET[i] as u8 == letter {
            return i as i8
        }
    }

    return -1
}

/// returns the position in the unicode-table
pub fn pos_in_unicode(letter: u16, unicode_table: &Vec<u16>) -> i8 {
    for (i, character) in unicode_table.iter().enumerate() {
        if *character == letter {
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
fn pos_to_index(position: usize) -> usize {
    2 * (position / 3)
}

#[test]
fn test_pos_in_alpha() {
    assert_eq!(pos_in_alpha('a' as u8), 0);
    assert_eq!(pos_in_alpha('b' as u8), 1);
    assert_eq!(pos_in_alpha('c' as u8), 2);
    assert_eq!(pos_in_alpha('A' as u8), 26);
    assert_eq!(pos_in_alpha('B' as u8), 27);
    assert_eq!(pos_in_alpha('C' as u8), 28);
}

#[test]
fn test_pos_to_index() {
    assert_eq!(pos_to_index(5), 2);
}

#[test]
fn test_shift() {
    assert_eq!(shift(6,2), 6);
    assert_eq!(shift(26,5), 26);
}

#[test]
fn test_string_to_zchar() {
    let mut vec: Vec<u16> = Vec::new();
    assert_eq!(string_to_zchar("i am a string, please test me, no unicode",&vec), vec![14, 0, 6, 18, 0, 6, 0, 24, 25, 23, 14, 19, 12, 5, 19, 0, 21, 17, 10, 6, 24, 10, 0, 25, 10, 24, 25, 0, 18, 10, 5, 19, 0, 19, 20, 0, 26, 19, 14, 8, 20, 9, 10]);
    vec.push('€' as u16);
    assert_eq!(string_to_zchar("nasty char: €",&vec), vec![19, 6, 24, 25, 30, 0, 8, 13, 6, 23, 5, 29, 0, 5, 6, 4, 27]);
}