//! The `zfile` module contains functionality to create a zcode file
//! 

pub use super::zbytes::Bytes;
pub use super::ztext;
pub use super::op;

#[derive(Debug)]
pub enum ZOP {
  PrintUnicode{c: u16},
  PrintUnicodeVar{var: u8},
  Print{text: String},
  PrintNumVar{variable: u8},
  PrintPaddr{address: u8},  // address is a variable
  PrintPaddrStatic{address: u16},
  PrintAddr{address: u8},
  PrintOps{text: String},
  Call1N{jump_to_label: String},
  Call2NWithAddress{jump_to_label: String, address: String},
  Call2NWithLargeConst{jump_to_label: String, arg: u16},
  Call1NVar{variable: u8},
  Routine{name: String, count_variables: u8},
  Label{name: String},
  Newline,
  SetColor{foreground: u8, background: u8},
  SetColorVar{foreground: u8, background: u8},
  SetTextStyle{bold: bool, reverse: bool, monospace: bool, italic: bool},
  StoreU16{variable: u8, value: u16},
  StoreU8{variable: u8, value: u8},
  StoreW{array_address: u16, index: u8, variable: u8},
  Inc{variable: u8},
  Ret{value: u16},
  JE{local_var_id: u8, equal_to_const: i16, jump_to_label: String},
  Random{range: u8, variable: u8},
  ReadChar{local_var_id: u8},
  ReadCharTimer{local_var_id: u8, timer: u8, routine: String},
  Add{variable1: u8, add_const: i16, variable2: u8},
  AddVar{variable1: u8, variable2: u8, result: u8},
  Sub{variable1: u8, sub_const: i16, variable2: u8},
  JL{local_var_id: u8, local_var_id2: u8, jump_to_label: String},
  Jump{jump_to_label: String},
  Dec{variable: u8},
  LoadW{array_address: u16, index: u8, variable: u8},
  LoadWvar{array_address_var: u8, index: u8, variable: u8},
  EraseWindow{value: i8},
  Quit,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JumpType {
    Jump,
    Branch,
    Routine
}

/// types of possible arguments
pub enum ArgType {
    LargeConst,
    SmallConst,
    Variable,
    Reference,
    Nothing
}

pub struct Zfile {
    pub data: Bytes,
    unicode_table: Vec<u16>,
    jumps: Vec<Zjump>,
    labels: Vec<Zlabel>,
    strings: Vec<Zstring>,
    program_addr: u16,
    unicode_table_addr: u16,
    global_addr: u16,
    object_addr: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Zjump {
    pub from_addr: u16,
    pub name: String,
    pub jump_type: JumpType
}

#[derive(Debug, PartialEq, Clone)]
pub struct Zstring {
    pub from_addr: u16,
    pub chars: Vec<u8>,  // contains either ztext or [length:u16, utf16char:u16, …]
    pub orig: String,
    pub unicode: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Zlabel {
    pub to_addr: u16,
    pub name: String
}

#[derive(Debug, Copy, Clone)]
pub struct FormattingState {
    pub bold: bool,
    pub mono: bool,
    pub italic: bool,
    pub inverted: bool
}


impl Zfile {

    /// creates a new zfile
    pub fn new() -> Zfile {
        Zfile {
            data: Bytes{bytes: Vec::new()},
            unicode_table: Vec::new(),
            jumps: Vec::new(),
            labels: Vec::new(),
            strings: Vec::new(),
            program_addr: 0x800,
            unicode_table_addr: 0,
            global_addr: 0,
            object_addr: 0,
        }
    }

    /// creates the header of a zfile
    pub fn create_header(&mut self) {
        
        assert!(self.data.len() == 0, "create_header should run at the beginning of the op-codes");

        let alpha_addr: u16 = 0x40;
        let extension_addr: u16 = alpha_addr + 78;
        self.unicode_table_addr = extension_addr as u16 + 4;

        // 1 byte for the unicode count, 97 possible chars with 2 bytes
        self.global_addr = self.unicode_table_addr + 195;

        // 480 becouse there are 240 global 2-bytes variables
        self.object_addr = self.global_addr + 480;
        let high_memory_addr: u16 = 0x800;
        let static_addr: u16 = 0x800;
        let dictionary_addr: u16 = 0x800;

        // version
        self.data.write_byte(8, 0x00);

        // flag1 (from right to left)
        // 0: colours availabe
        // 1: picture
        // 2: bold
        // 3: italic
        // 4: fixed
        self.data.write_byte(0x1d, 0x01);

        // release version (0x02 und 0x03)
        self.data.write_u16(0, 0x02);

        // base of high memory (byte address) (0x04 and 0x05)
        self.data.write_u16(high_memory_addr, 0x04);

        // initial value of programm counter (0x06 and 0x07)
        self.data.write_u16(self.program_addr, 0x06);

        // location of dictionary (byte address) (0x08 and 0x09)
        self.data.write_u16(dictionary_addr, 0x08);

        // flag2 (from right to left)
        // 6: game want to use colours
        // 0000000001000000
        self.data.write_u16(0x40, 0x10);

        // location of object table (byte address) (0x0a and 0x0b)
        self.data.write_u16(self.object_addr, 0x0a);

        // location of global variables table (byte address) (0x0c and 0x0d)
        self.data.write_u16(self.global_addr, 0x0c);

        // base of static memory (byte address) (0x0e and 0x0f)
        self.data.write_u16(static_addr, 0x0e);

        // alphabet address (bytes) - its 0x34 and 0x35, why not only 0x34?
        self.data.write_u16(alpha_addr, 0x34);

        // header extension table address (bytes) - its 0x36 and 0x37, why not only 0x36?
        self.data.write_u16(extension_addr, 0x36);

        // alphabet
        self.write_alphabet(alpha_addr as usize);

        // header extension table
        self.data.write_u16(3, extension_addr as usize);     // Number of further words in table
        self.data.write_u16(0, extension_addr as usize + 1); // x-coordinate of mouse after a click
        self.data.write_u16(0, extension_addr as usize + 2); // y-coordinate of mouse after a click
        self.data.write_u16(0, extension_addr as usize + 3); // if != 0: unicode translation table address (optional)

        // global variables
        // ...
    }

    /// writes the alphabet to index
    fn write_alphabet(&mut self, index: usize) {
        // TODO: is it possible to do this with map?
        let mut alpha_tmp: [u8; 78] = [0; 78];
        for i in 0..ztext::ALPHABET.len() {
            alpha_tmp[i] = ztext::ALPHABET[i] as u8;
        }
        self.data.write_bytes(&alpha_tmp, index);
    }

    /// writes the unicode table to the address unicode_table_addr
    fn write_unicode_table(&mut self) {
        self.data.write_byte(self.unicode_table.len() as u8, self.unicode_table_addr as usize);

        for (i, character) in self.unicode_table.iter().enumerate() {
            self.data.write_u16(*character, self.unicode_table_addr as usize + 1 + 2*i);
        }

    }

    /// saves the addresses of the labels to the positions of the jump-ops
    /// goes through all jumps and labels, if they have the same name:
    ///  write the "where to jump"-adress of the label to the position of the jump
    fn write_jumps(&mut self) {
        for jump in self.jumps.iter_mut() {
            let mut label_found = false;

            for label in self.labels.iter_mut() {
                if label.name == jump.name {
                    label_found = true;
                    match jump.jump_type {
                        JumpType::Routine => {
                            let new_addr: u16 = label.to_addr / 8;
                            self.data.write_u16(new_addr, jump.from_addr as usize);
                        },
                        JumpType::Branch => {
                            let mut new_addr: i32 = label.to_addr as i32 - jump.from_addr as i32;
                            new_addr &= 0x3fff;
                            new_addr |= 0x8000;
                            self.data.write_u16(new_addr as u16, jump.from_addr as usize);
                        },
                        JumpType::Jump => {
                            let new_addr: i32 = label.to_addr as i32 - jump.from_addr as i32;
                            self.data.write_u16(new_addr as u16, jump.from_addr as usize);
                        }
                    }
                }
            }

            if label_found == false {
                panic!("Should generate jump to label \"{}\" but no such label exists", jump.name);
            }
        }
    }

    /// saves the zstrings to high mem and writes the resulting address to the
    /// print_paddr arguments which referencing the string
    fn write_strings(&mut self) {
        let mut prev_strings: Vec<(Zstring, u16)> = vec![];
        for string in self.strings.iter_mut() {
            // optimize to reuse strings if they are the same
            let mut string_found = false;
            for &(ref other, addr) in prev_strings.iter() {
                if other.orig == string.orig && other.unicode == string.unicode {
                    string_found = true;
                    self.data.write_u16(addr/8 as u16, string.from_addr as usize);
                    break;
                }
            }
            if string_found == false {  // add new string to high mem
                let str_addr = align_address(self.data.len() as u16, 8);
                self.data.write_zero_until(str_addr as usize);
                if string.unicode {
                    debug!("{:#x}: utf16 \"{}\"", str_addr, string.orig);
                    let hexstrs: Vec<String> = string.chars.iter().map(|b| format!("{:02X}", b)).collect();
                    trace!("{:#x}: {}", str_addr, hexstrs.connect(" "));
                    self.data.append_bytes(&string.chars);
                    self.data.write_u16(str_addr/8 as u16, string.from_addr as usize);
                } else {
                    debug!("{:#x}: zstring \"{}\"", str_addr, string.orig);
                    let hexstrs: Vec<String> = string.chars.iter().map(|b| format!("{:02X}", b)).collect();
                    trace!("{:#x}: {}", str_addr, hexstrs.connect(" "));
                    self.data.append_bytes(&string.chars);
                    self.data.write_u16(str_addr/8 as u16, string.from_addr as usize);
                }
                prev_strings.push((string.clone(), str_addr));
            }
        }
    }

    /// adds jump to write the jump-addresses after reading all commands
    pub fn add_jump(&mut self, name: String, jump_type: JumpType) {
        let from_addr: u16 = self.data.bytes.len() as u16;
        let jump: Zjump = Zjump{ from_addr: from_addr, name: name, jump_type: jump_type};
        self.jumps.push(jump);

        // spacer for the adress where the to-jump-label will be written
        self.data.write_u16(0x0000, from_addr as usize);
    }

    /// adds label to the labels-vector. we need them later
    fn add_label(&mut self, name: String, to_addr: u16) {
        let label: Zlabel = Zlabel{ name: name, to_addr: to_addr };
        for other_label in self.labels.iter() {
            if other_label.name == label.name {
                panic!("label has to be unique, but \"{}\" isn't.", other_label.name);
            }
        }
        self.labels.push(label);
    }

    /// write out respective byte stream of opcodes to file
    pub fn emit(&mut self, code: Vec<ZOP>) {
        for instr in &code {
            let addr = self.data.bytes.len();
            debug!("{:#x}: {:?}", addr, instr);
            let (_, _, bytes) = self.write_zop(instr);
            let hexstrs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            trace!("{:#x}: {}", addr, hexstrs.connect(" "));
        }
    }

    /// write opcodes to file but also return written bytes for testing purposes
    /// as well as the resulting new labels and jumps
    pub fn write_zop(&mut self, instr: &ZOP) -> (Vec<Zlabel>, Vec<Zjump>, Vec<u8>){
        let beginning = self.data.bytes.len();
        let old_jumps: Vec<Zjump> = self.jumps.clone();
        let old_labels: Vec<Zlabel> = self.labels.clone();


        //self.data.write_bytes()
        let bytes: Vec<u8> = match instr {
            &ZOP::Quit => op::quit(),
            &ZOP::Newline => op::op_newline(),
            &ZOP::Dec{variable} => op::op_dec(variable),
            &ZOP::Inc{variable} => op::op_inc(variable),
            &ZOP::Add{variable1, add_const, variable2} => op::op_add(variable1, add_const, variable2),
            &ZOP::AddVar{variable1, variable2, result} => op::op_add_var(variable1, variable2, result),
            &ZOP::Sub{variable1, sub_const, variable2} => op::op_sub(variable1, sub_const, variable2),
            &ZOP::StoreU16{variable, value} => op::op_store_u16(variable, value),
            &ZOP::StoreU8{variable, value} => op::op_store_u8(variable, value),
            &ZOP::Ret{value} => op::op_ret(value),
            &ZOP::PrintAddr{address} => op::op_print_addr(address),
            &ZOP::PrintPaddr{address} => op::op_print_paddr(address),
            &ZOP::PrintPaddrStatic{address} => op::op_print_paddr_static(address),
            &ZOP::SetColor{foreground, background} => op::op_set_color(foreground, background),
            &ZOP::SetColorVar{foreground, background} => op::op_set_color_var(foreground, background),
            &ZOP::Random{range, variable} => op::op_random(range, variable),
            &ZOP::PrintNumVar{variable} => op::op_print_num_var(variable),
            &ZOP::SetTextStyle{bold, reverse, monospace, italic} => op::op_set_text_style(bold, reverse, monospace, italic),
            &ZOP::ReadChar{local_var_id} => op::op_read_char(local_var_id),
            &ZOP::LoadW{array_address, index, variable} => op::op_loadw(array_address, index, variable, self.object_addr),
            &ZOP::LoadWvar{array_address_var, index, variable} => op::op_loadw_var(array_address_var, index, variable),
            &ZOP::StoreW{array_address, index, variable} => op::op_storew(array_address, index, variable,self.object_addr),
            &ZOP::Call1NVar{variable} => op::op_call_1n_var(variable),
            &ZOP::EraseWindow{value} => op::op_erase_window(value),

            _ => Vec::new()
        };
        self.data.append_bytes(&bytes);
        match instr {
            &ZOP::PrintUnicode{c} => self.op_print_unicode_char(c),
            &ZOP::PrintUnicodeVar{var} => self.op_print_unicode_var(var),
            &ZOP::Print{ref text} => self.op_print(text),
            &ZOP::PrintOps{ref text} => self.gen_print_ops(text),
            &ZOP::Routine{ref name, count_variables} => self.routine(name, count_variables),
            &ZOP::Label{ref name} => self.label(name),
            &ZOP::Jump{ref jump_to_label} => self.op_jump(jump_to_label),
            &ZOP::ReadCharTimer{local_var_id, timer, ref routine} => self.op_read_char_timer(local_var_id, timer, routine),
            &ZOP::JL{local_var_id, local_var_id2, ref jump_to_label} => self.op_jl(local_var_id, local_var_id2, jump_to_label),
            &ZOP::JE{local_var_id, equal_to_const, ref jump_to_label} => self.op_je(local_var_id, equal_to_const, jump_to_label),
            &ZOP::Call2NWithAddress{ref jump_to_label, ref address} => self.op_call_2n_with_address(jump_to_label, address),
            &ZOP::Call2NWithLargeConst{ref jump_to_label, arg} => self.op_call_2n_with_large_const(jump_to_label, arg),
            &ZOP::Call1N{ref jump_to_label} => self.op_call_1n(jump_to_label),
            _ => ()
        }
        let mut new_jumps: Vec<Zjump> = vec![];
        let mut new_labels: Vec<Zlabel> = vec![];
        for label in self.labels.iter() {
            if !old_labels.contains(&label) {
                new_labels.push(label.clone());
            }
        }
        for jump in self.jumps.iter() {
            if !old_jumps.contains(&jump) {
                new_jumps.push(jump.clone());
            }
        }
        (new_labels, new_jumps, self.data.bytes[beginning..self.data.bytes.len()].to_vec())
    }

    /// generates normal print opcodes for ASCII characters and unicode print
    /// opcodes for unicode characters
    pub fn gen_print_ops(&mut self, text: &str) {
        let mut current_text: String = String::new();
        let mut current_utf16: String = String::new();
        for character in text.chars() {
            if character as u32 <= 126 {
                self.gen_write_out_unicode(current_utf16.to_string());  // write out utf16 string
                current_utf16.clear();
                // this is a non-unicode char
                current_text.push(character);

            } else if character as u32 > 0xFFFF {
                self.gen_write_out_unicode(current_utf16.to_string());  // write out utf16 string
                current_utf16.clear();
                // zcode has no support for such high unicode values
                current_text.push('?');
            } else {
                if ztext::pos_in_unicode(character as u16, &self.unicode_table) != -1 {
                    self.gen_write_out_unicode(current_utf16.to_string());  // write out utf16 string
                    current_utf16.clear();
                    // unicode exist in table
                    current_text.push(character);
                } else if self.unicode_table.len() < 97 {
                    self.gen_write_out_unicode(current_utf16.to_string());  // write out utf16 string
                    current_utf16.clear();
                    // there is space in the unicode table
                    trace!("added char '{:?}' to unicode_table", character);
                    self.unicode_table.push(character as u16);
                    current_text.push(character);
                } else {
                    // no space, so op_print_unicode_char is the answer
                    trace!("Unicode char '{:?}' is not in unicode_table", character.to_string());
                    if current_text.len() > 0 {
                        if current_text.len() > 3 {  // write string to high mem
                            self.gen_high_mem_zprint(&current_text[..]);
                            current_text.clear();
                        } else { // print in place
                            self.emit(vec![ZOP::Print{text: current_text.to_string()}]);
                            current_text.clear();
                        }
                    }
                    current_utf16.push(character);
                }
            }
        }
        self.gen_write_out_unicode(current_utf16);  // write out utf16 string
        if current_text.len() > 0 {
            if current_text.len() > 3 {  // write string to high mem
                self.gen_high_mem_zprint(&current_text[..]);
            } else {  // print in place
                self.emit(vec![ZOP::Print{text: current_text.to_string()}]);
            }
        }
    }

    fn gen_write_out_unicode(&mut self, current_utf16: String) {
        if current_utf16.len() > 0 {
            if current_utf16.len() > 1 {
                let mut utf16bytes: Vec<u8> = vec![];
                for c in current_utf16.chars() {
                    let value: u16 = c as u16;
                    utf16bytes.push((value >> 8) as u8);
                    utf16bytes.push((value & 0xff) as u8);
                }
                let length: u16 = utf16bytes.len() as u16 / 2u16;
                utf16bytes.insert(0, (length >> 8) as u8);
                utf16bytes.insert(1, (length & 0xff) as u8);
                self.emit(vec![ZOP::Call2NWithLargeConst{jump_to_label: "print_unicode".to_string(), arg: 0u16}]);
                self.strings.push(Zstring{chars: utf16bytes, orig: current_utf16.to_string(), from_addr: (self.data.len()-2) as u16, unicode: true});
            } else {
                self.emit(vec![ZOP::PrintUnicode{c: current_utf16.chars().nth(0).unwrap() as u16}]);
            }
        }
    }

    fn gen_high_mem_zprint(&mut self, text: &str) {
        self.emit(vec![ZOP::PrintPaddrStatic{address: 0u16}]);  // dummy addr
        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        ztext::encode(&mut text_bytes, text, &self.unicode_table);
        self.strings.push(
            Zstring{chars: text_bytes.bytes, orig: text.to_string(), from_addr: (self.data.len()-2) as u16, unicode: false});
    }

    // ================================
    // no op-commands

    /// start of an zcode programm
    /// fills everying < program_addr with zeros
    /// should called as the first commend
    pub fn start(&mut self) {
        self.create_header();
        self.data.write_zero_until(self.program_addr as usize);

        // default theme and erase_window to fore the color
        self.emit(vec![
            ZOP::SetColor{foreground: 9, background: 2},
            ZOP::EraseWindow{value: -1},
            ZOP::Call1N{jump_to_label: "Start".to_string()},
        ]);
    }

    /// writes all stuff that couldn't written directly
    /// should be called as the last commend
    pub fn end(&mut self) {
        self.write_unicode_table();
        self.routine_check_links();
        self.routine_add_link();
        self.routine_check_more();
        self.routine_print_unicode();
        self.write_jumps();
        self.write_strings();
    }

    /// command to create a routine
    pub fn routine(&mut self, name: &str, count_variables: u8) {    
        let index: u16 = routine_address(self.data.bytes.len() as u16);
        
        assert!(count_variables <= 15, "only 15 local variables are allowed");
        assert!(index % 8 == 0, "adress of a routine must start at address % 8 == 0");

        self.add_label(name.to_string(), index);
        self.data.write_byte(count_variables, index as usize);
    }

    /// command to create a label
    pub fn label(&mut self, name: &str) {
        let index: usize = self.data.bytes.len();
        self.add_label(name.to_string(), index as u16);
    }

    // ================================
    // zcode routines

    /// routine to add the address of a passage-link
    pub fn routine_add_link(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "system_add_link".to_string(), count_variables: 1},
            // saves routine-argument to array
            ZOP::StoreW{array_address: 1, index: 16, variable: 0x01},

            // inc the count links
            ZOP::Inc{variable: 16},

            ZOP::Ret{value: 0}
        ]);
    }

    /// exits the program
    /// quit is 0OP
    pub fn op_quit(&mut self) {
        self.op_0(0x0a);
    }


    /// checks all stored links and make them choiceable
    /// with the keyboard
    pub fn routine_check_links(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "system_check_links".to_string(), count_variables: 2},

            // jumps to the end, if this passage was called as <<display>>
            ZOP::JE{local_var_id: 17, equal_to_const: 0x01, jump_to_label: "system_check_links_end_ret".to_string()},

            // jumps to the end, if there a no links
            ZOP::JE{local_var_id: 16, equal_to_const: 0x00, jump_to_label: "system_check_links_end_quit".to_string()},
            ZOP::Print{text: "--------------------".to_string()},
            ZOP::Newline,
            ZOP::Print{text: "press a key... ".to_string()},
            ZOP::Newline,

            ZOP::Label{name: "system_check_links_loop".to_string()},
            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 129, jump_to_label: "system_check_links_jmp".to_string()},
            ZOP::Jump{jump_to_label: "system_check_links_after".to_string()},
            ZOP::Label{name: "system_check_links_jmp".to_string()},
            ZOP::Call1N{jump_to_label: "system_check_more".to_string()},

            ZOP::Label{name: "system_check_links_after".to_string()},

            ZOP::Sub{variable1: 0x01, sub_const: 48, variable2: 0x01},

            // check if the link in 0x01 exist, if not
            // => "wrong key => jump before key-detection
            ZOP::JL{local_var_id: 16, local_var_id2: 0x01, jump_to_label: "system_check_links_loop".to_string()},

            // check if the key-48 is < 0, if it is => jump before key-detection
            ZOP::StoreU8{variable: 0x02, value: 1},
            ZOP::JL{local_var_id: 0x01, local_var_id2: 0x02, jump_to_label: "system_check_links_loop".to_string()},
            ZOP::Dec{variable: 0x01},

            // loads the address of the link from the array
            ZOP::LoadW{array_address: 1, index: 0x01, variable: 0x02},

            // no more links exist
            ZOP::StoreU8{variable: 16, value: 0},
            ZOP::Newline,

            // clears window bevor jumping
            ZOP::EraseWindow{value: -1},

            // jump to the new passage
            ZOP::Call1NVar{variable: 0x02},
            ZOP::Label{name: "system_check_links_end_ret".to_string()},
            ZOP::Ret{value: 0},

            ZOP::Label{name: "system_check_links_end_quit".to_string()},
            ZOP::Quit
        ]);
    }

    /// easter-egg, with konami-code to start
    pub fn routine_check_more(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "system_check_more".to_string(), count_variables: 1},
            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 129, jump_to_label: "system_check_more_ko_1".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_1".to_string()},
        
            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 130, jump_to_label: "system_check_more_ko_2".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_2".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 130, jump_to_label: "system_check_more_ko_3".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_3".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 131, jump_to_label: "system_check_more_ko_4".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_4".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 132, jump_to_label: "system_check_more_ko_5".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_5".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 131, jump_to_label: "system_check_more_ko_6".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_6".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 132, jump_to_label: "system_check_more_ko_7".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_7".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 98, jump_to_label: "system_check_more_ko_8".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_8".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{local_var_id: 0x01, equal_to_const: 97, jump_to_label: "system_check_more_ko_9".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_9".to_string()},

            ZOP::Label{name: "system_check_more_timer_loop".to_string()},
            ZOP::ReadCharTimer{local_var_id: 0x01, timer: 1, routine: "system_check_more_anim".to_string()},
            ZOP::Jump{jump_to_label: "system_check_more_timer_loop".to_string()},
            ZOP::Quit,

            ZOP::Routine{name: "system_check_more_anim".to_string(), count_variables: 5},
            ZOP::EraseWindow{value: -1},

            ZOP::SetTextStyle{bold: false, reverse: false, monospace: true, italic: false},
            ZOP::SetColor{foreground: 2, background: 9},
            ZOP::Print{text: " ZWREEC Easter egg <3".to_string()},
            ZOP::Newline,

            ZOP::StoreU8{variable: 1, value: 20},
            ZOP::Label{name: "system_check_more_loop".to_string()},
            ZOP::Random{range: 8, variable: 4},
            ZOP::Random{range: 100, variable: 5},
            ZOP::Add{variable1: 5, add_const: 10, variable2: 5},
            ZOP::Inc{variable: 4},
            ZOP::SetColorVar{foreground: 4, background: 4},
            ZOP::Print{text: "aa".to_string()},
            ZOP::Inc{variable: 2},

            ZOP::JL{local_var_id: 2, local_var_id2: 1, jump_to_label: "system_check_more_loop".to_string()},
            ZOP::Newline,
            ZOP::Inc{variable: 3},
            ZOP::StoreU8{variable: 2, value: 0},
            ZOP::JL{local_var_id: 3, local_var_id2: 1, jump_to_label: "system_check_more_loop".to_string()},
            ZOP::Ret{value: 0}
        ]);
    }

    /// print UTF-16 string from addr
    /// expects an address as argument where first the length (as u16)
    /// of the string is written (number of u16 chars), followed by the string to print.
    /// only works if the string is within the address space up to 0xffff
    pub fn routine_print_unicode(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "print_unicode".to_string(), count_variables: 3},
            // addr in 0x01, length in 0x02
            ZOP::LoadWvar{array_address_var: 0x01, index: 0, variable: 0x02},
            ZOP::AddVar{variable1: 0x02, variable2: 0x02, result: 0x02}, // double length
            ZOP::AddVar{variable1: 0x01, variable2: 0x02, result: 0x02}, // add 'offset' addr to length,
            // so it marks the position after the last char
            ZOP::Add{variable1: 0x01, add_const: 2i16, variable2: 0x01}, // point to first char
            ZOP::Label{name: "inter_char".to_string()},
            // load u16 char to 0x3
            ZOP::LoadWvar{array_address_var: 0x01, index: 0, variable: 0x03},
            ZOP::PrintUnicodeVar{var: 0x03},
            ZOP::Add{variable1: 0x01, add_const: 2i16, variable2: 0x01}, // point to next char
            ZOP::JL{local_var_id: 0x01, local_var_id2: 0x02, jump_to_label: "inter_char".to_string()},
            ZOP::Ret{value: 0}
        ]);
    }



    // ================================
    // specific ops

    /// print strings
    /// print is 0OP
    fn op_print(&mut self, content: &str) {
        let index: usize = self.data.bytes.len();
        self.op_0(0x02);

        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        ztext::encode(&mut text_bytes, content, &self.unicode_table);
        self.data.write_bytes(&text_bytes.bytes, index + 1);
    }

    /// jumps to a label
    pub fn op_jump(&mut self, jump_to_label: &str) {
        self.op_1(0x0c, ArgType::LargeConst);
        self.add_jump(jump_to_label.to_string(), JumpType::Jump);
    }



    /// calls a routine
    /// call_1n is 1OP
    pub fn op_call_1n(&mut self, jump_to_label: &str) {
        self.op_1(0x0f, ArgType::LargeConst);
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);
    }


    /// calls a routine with an argument(variable) an throws result away
    /// becouse the value isn't known until all routines are set, it
    /// inserts a pseudo routoune_address
    /// call_2n is 2OP
    pub fn op_call_2n_with_address(&mut self, jump_to_label: &str, address: &str) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::LargeConst];
        self.op_2(0x1a, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        // the address of the argument
        self.add_jump(address.to_string(), JumpType::Routine);
    }

    /// calls a routine with one argument (here LargeConst) an throws result away
    /// call_2n is 2OP
    pub fn op_call_2n_with_large_const(&mut self, jump_to_label: &str, arg: u16) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::LargeConst];
        self.op_2(0x1a, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        self.data.append_u16(arg); // just one parameter
    }


    /// jumps to a label if the value of local_var_id is equal to const
    /// is an 2OP, but with small constant and variable
    pub fn op_je(&mut self, local_var_id: u8, equal_to_const: i16, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::LargeConst];
        self.op_2(0x01, args);
        
        // variable id
        self.data.append_byte(local_var_id);

        // const
        self.data.append_u16(equal_to_const as u16);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::Branch);
    }


    /// jumps to a label if the value of local_var_id is smaller than local_var_id2 (compared as i16)
    /// is an 2OP, but with variable and variable
    pub fn op_jl(&mut self, local_var_id: u8, local_var_id2: u8, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::Variable];
        self.op_2(0x02, args);

        // variable id
        self.data.append_byte(local_var_id);

        // variable id 2
        self.data.append_byte(local_var_id2);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::Branch);
    }


    /// reads keys from the keyboard and saves the asci-value in local_var_id
    /// read_char is VAROP
    pub fn op_read_char_timer(&mut self, local_var_id: u8, timer: u8, routine: &str) {
        let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::SmallConst, ArgType::LargeConst, ArgType::Nothing];
        self.op_var(0x16, args);

        // write argument value
        self.data.append_byte(0x00);

        // write timer
        self.data.append_byte(timer);

        // writes routine
        self.add_jump(routine.to_string(), JumpType::Routine);

        // write varible id
        self.data.append_byte(local_var_id);
    }


    /// prints an unicode char to the current stream
    pub fn op_print_unicode_char(&mut self, character: u16) {

        self.op_1(0xbe, ArgType::SmallConst);
        self.data.append_byte(0x0b);
        // 0x00 means LargeConst, then 0x03 means omitted, 0x02 means Variable, 0x01 means SmallConst
        let byte = 0x00 << 6 | 0x03 << 4 | 0x03 << 2 | 0x03 << 0;
        self.data.append_byte(byte);
        self.data.append_u16(character);
    }

    /// prints variable with unicode char to the current stream
    pub fn op_print_unicode_var(&mut self, variable: u8) {

        self.op_1(0xbe, ArgType::SmallConst);
        self.data.append_byte(0x0b);
        // 0x00 means LargeConst, then 0x03 means omitted, 0x02 means Variable, 0x01 means SmallConst
        let byte = 0x02 << 6 | 0x03 << 4 | 0x03 << 2 | 0x03 << 0;
        self.data.append_byte(byte);
        self.data.append_byte(variable);
    }

    // ================================
    // general ops

    /// op-codes with 0 operators
    fn op_0(&mut self, value: u8) {
        self.data.append_bytes(&op::op_0(value));
    }
    
    /// op-codes with 1 operator
    fn op_1(&mut self, value: u8, arg_type: ArgType) {
        self.data.append_bytes(&op::op_1(value, arg_type));
    }

    /// op-codes with 1 operator
    fn op_2(&mut self, value: u8, arg_types: Vec<ArgType>) {
        self.data.append_bytes(&op::op_2(value, arg_types));
    }

    fn op_var(&mut self, value: u8, arg_types: Vec<ArgType>) {
        self.data.append_bytes(&op::op_var(value, arg_types));
    }
}

fn align_address(address: u16, align: u16) -> u16 {
    address + (align - (address % align)) % align
}

/// returns the routine address, should be adress % 8 == 0 (becouse its an packed address)
fn routine_address(address: u16) -> u16 {
    return align_address(address, 8);
}

// ================================
// test functions

#[test]
fn test_align_address() {
    assert_eq!(align_address(0xf, 8), 0x10);
    assert_eq!(align_address(0x7, 8), 0x8);
    assert_eq!(align_address(0x8, 8), 0x8);
    assert_eq!(align_address(0x9, 8), 0x10);
    assert_eq!(align_address(0x10, 16), 0x10);
    assert_eq!(align_address(0x1f, 32), 0x20);
    assert_eq!(align_address(0x20, 32), 0x20);
    assert_eq!(align_address(0x21, 32), 0x40);
}

#[test]
fn test_routine_address() {
    assert_eq!(routine_address(8), 8);
    assert_eq!(routine_address(9), 16);
    assert_eq!(routine_address(10), 16);
    assert_eq!(routine_address(15), 16);
    assert_eq!(routine_address(17), 24);
}

#[test]
fn test_zfile_write_jumps_length() {
    let mut zfile: Zfile = Zfile::new();
    zfile.write_jumps();
    assert_eq!(zfile.data.len(), 0);
    
    zfile.op_jump("labename");
    assert_eq!(zfile.data.len(), 3);

    zfile.label("labename");
    zfile.write_jumps();
    assert_eq!(zfile.data.len(), 3);
}

#[test]
fn test_zfile_general_op_length() {
    let mut zfile: Zfile = Zfile::new();
    zfile.op_0(0x00);
    assert_eq!(zfile.data.len(), 1);
    zfile.op_1(0x00, ArgType::SmallConst);
    assert_eq!(zfile.data.len(), 2);
    zfile.op_1(0x00, ArgType::Reference);
    assert_eq!(zfile.data.len(), 3);
    let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
    zfile.data.append_bytes(&op::op_var(0x00, args));
    assert_eq!(zfile.data.len(), 5);
}

#[test]
fn test_zfile_label_and_jump_loop() {
    let mut zfile: Zfile = Zfile::new();
    zfile.start();
    let (labels, jumps1, bytes1) =  zfile.write_zop(&ZOP::Label{name: "Start".to_string()});
    assert_eq!(jumps1.len() + bytes1.len(), 0);
    assert_eq!(labels.len(), 1);
    let (labels2, jumps, bytes) =  zfile.write_zop(&ZOP::Jump{jump_to_label: "Start".to_string()});
    assert_eq!(labels2.len(), 0);
    assert_eq!(jumps.len(), 1);
    assert_eq!(bytes.len(), 3);
    let pos = zfile.data.len() - bytes.len();  // start position of written bytes
    zfile.end();
    // in this example we have the following data:
    //[Zlabel { to_addr: 2055, name: "Start" }] [] []
    //[] [Zjump { from_addr: 2056, name: "Start", jump_type: Jump }] [140, 255, 255]
    // 0xffff is -1 as i16 because we have a relative jump
    assert_eq!(zfile.data.bytes[pos], bytes[0]);  // jump op
    let rel_addr: i16 = (zfile.data.bytes[pos+1] as u16 * 256 + zfile.data.bytes[pos+2] as u16) as i16;
    assert_eq!((labels[0].to_addr as i32 - jumps[0].from_addr as i32) as i16, rel_addr);  // specified as in write_jumps()
    assert_eq!(-1 as i16, rel_addr);  // this is the expected result, jump one address back
}
