//! The `zfile` module contains functionality to create a zcode file
//! 
pub use super::zbytes::Bytes;
pub use super::ztext;
pub use super::op;

#[derive(Debug,Clone)]
pub struct Variable { pub id: u8 }
#[derive(Debug,Clone)]
pub struct Constant { pub value: u8 }
#[derive(Debug,Clone)]
pub struct LargeConstant { pub value: i16 }

#[derive(Debug,Clone)]
pub enum Operand {
    Var(Variable),
    Const(Constant) ,
    LargeConst(LargeConstant),
    StringRef(LargeConstant),
}

impl Operand {
    pub fn new_const(value: u8) -> Operand {
        Operand::Const(Constant { value: value })
    }

    pub fn new_large_const(value: i16) -> Operand {
        Operand::LargeConst(LargeConstant { value: value })
    }

    pub fn new_string_ref(value: i16) -> Operand {
        Operand::StringRef(LargeConstant { value: value })
    }

    pub fn new_var(id: u8) -> Operand {
        Operand::Var(Variable::new(id))
    }

    pub fn const_value(&self) -> i16 {
        match self {
            &Operand::Const(ref constant) => constant.value as i16,
            &Operand::LargeConst(ref constant) => constant.value,
            _ => panic!("Operand must be a constant!")
        }
    }

    pub fn is_const(&self) -> bool {
        match self {
            &Operand::Const(_) | &Operand::LargeConst(_) => true,
            _ => false
        }
    }
}

impl Variable {
    pub fn new(id: u8) -> Variable {
        Variable { id: id }
    }
}

#[derive(Debug)]
pub enum ZOP {
  PrintUnicode{c: u16},
  PrintUnicodeVar{var: Variable},
  PrintUnicodeStr{address: Operand},
  Print{text: String},
  PrintNumVar{variable: Variable},
  PrintPaddr{address: Operand},  // packed address
  PrintAddr{address: Operand},
  PrintOps{text: String},
  Call1N{jump_to_label: String},
  Call2NWithAddress{jump_to_label: String, address: String},
  Call2NWithArg{jump_to_label: String, arg: Operand},
  Call1NVar{variable: u8},
  Routine{name: String, count_variables: u8},
  Label{name: String},
  Newline,
  SetColor{foreground: u8, background: u8},
  SetColorVar{foreground: u8, background: u8},
  SetTextStyle{bold: bool, reverse: bool, monospace: bool, italic: bool},
  StoreVariable{variable: Variable, value: Operand},
  StoreW{array_address: Operand, index: Variable, variable: Variable},
  Inc{variable: u8},
  Ret{value: u16},
  JE{operand1: Operand, operand2: Operand, jump_to_label: String},
  JL{operand1: Operand, operand2: Operand, jump_to_label: String},
  JLE{operand1: Operand, operand2: Operand, jump_to_label: String},
  JG{operand1: Operand, operand2: Operand, jump_to_label: String},
  JGE{operand1: Operand, operand2: Operand, jump_to_label: String},
  Random{range: Operand, variable: Variable},
  ReadChar{local_var_id: u8},
  ReadCharTimer{local_var_id: u8, timer: u8, routine: String},
  Add{operand1: Operand, operand2: Operand, save_variable: Variable},
  Sub{operand1: Operand, operand2: Operand, save_variable: Variable},
  Mul{operand1: Operand, operand2: Operand, save_variable: Variable},
  Div{operand1: Operand, operand2: Operand, save_variable: Variable},
  Mod{operand1: Operand, operand2: Operand, save_variable: Variable},
  Or{operand1: Operand, operand2: Operand, save_variable: Variable},
  And{operand1: Operand, operand2: Operand, save_variable: Variable},
  Jump{jump_to_label: String},
  Dec{variable: u8},
  LoadW{array_address: Operand, index: Variable, variable: Variable},
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
    pub object_addr: u16,
    last_static_written: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Zjump {
    pub from_addr: u32,
    pub name: String,
    pub jump_type: JumpType
}

#[derive(Debug, PartialEq, Clone)]
pub struct Zstring {
    pub from_addr: u32,
    pub chars: Vec<u8>,  // contains either ztext or [length:u16, utf16char:u16, …]
    pub orig: String,
    pub unicode: bool,
    pub written_addr: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Zlabel {
    pub to_addr: u32,
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
            program_addr: 0xfff8,
            unicode_table_addr: 0,
            global_addr: 0,
            object_addr: 0,
            last_static_written: 0x800,
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

        // 480 because there are 240 global 2-bytes variables
        self.object_addr = self.global_addr + 480;
        let high_memory_addr: u16 = self.program_addr;
        let static_addr: u16 = self.last_static_written;
        let dictionary_addr: u16 = self.last_static_written;

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
                            let new_addr: u16 = (label.to_addr / 8) as u16;
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

    pub fn write_string(&mut self, newstring: &str) -> u16 {
        self.write_strings();
        for string in self.strings.iter_mut() {
            if string.orig == newstring && string.unicode {
                return string.written_addr as u16;
            }
        }
        let mut utf16bytes: Vec<u8> = vec![];
        for c in newstring.chars() {
            let value: u16 = c as u16;
            utf16bytes.push((value >> 8) as u8);
            utf16bytes.push((value & 0xff) as u8);
         }
        let length: u16 = utf16bytes.len() as u16 / 2u16;
        utf16bytes.insert(0, (length >> 8) as u8);
        utf16bytes.insert(1, (length & 0xff) as u8);
        let str_addr: u16 = self.last_static_written;
        assert!(str_addr >= self.object_addr && str_addr + (utf16bytes.len() as u16) < self.program_addr, "invalid addr to store a string");
        debug!("{:#x}: utf16 \"{}\"", str_addr, newstring);
        let hexstrs: Vec<String> = utf16bytes.iter().map(|b| format!("{:02X}", b)).collect();
        trace!("{:#x}: {}", str_addr, hexstrs.connect(" "));
        self.data.write_bytes(&utf16bytes, str_addr as usize);
        self.last_static_written = self.last_static_written + utf16bytes.len() as u16;
        self.strings.push(Zstring{orig: newstring.to_string(), chars: utf16bytes, unicode: true, written_addr: str_addr as u32, from_addr: 0});
        str_addr
    }

    /// saves the zstrings to high mem and writes the resulting address to the
    /// print_paddr arguments which referencing the string
    fn write_strings(&mut self) {
        let mut prev_strings: Vec<(Zstring, u32)> = vec![];
        for string in self.strings.iter_mut() {
            // optimize to reuse strings if they are the same
            let mut string_found = false;
            for &(ref other, addr) in prev_strings.iter() {
                if other.orig == string.orig && other.unicode == string.unicode {
                    string_found = true;
                    if string.unicode {
                        self.data.write_u16(addr as u16, string.from_addr as usize);  // normal addr
                    } else {
                        self.data.write_u16((addr/8) as u16, string.from_addr as usize);  // packed addr
                    }
                    break;
                }
            }
            if string_found == false {  // add new string to high mem
                let n_str_addr: u32 = if string.unicode && string.written_addr == 0 {
                    let str_addr: u16 = self.last_static_written;
                    assert!(str_addr >= self.object_addr && str_addr + (string.chars.len() as u16) < self.program_addr, "invalid addr to store a string");
                    debug!("{:#x}: utf16 \"{}\"", str_addr, string.orig);
                    let hexstrs: Vec<String> = string.chars.iter().map(|b| format!("{:02X}", b)).collect();
                    trace!("{:#x}: {}", str_addr, hexstrs.connect(" "));
                    self.data.write_bytes(&string.chars, str_addr as usize);
                    self.data.write_u16(str_addr as u16, string.from_addr as usize);  // normal addr
                    self.last_static_written = self.last_static_written + string.chars.len() as u16;
                    str_addr as u32
                } else if string.unicode == false && string.written_addr == 0 {
                    let str_addr: u32 = align_address(self.data.len() as u32, 8);
                    self.data.write_zero_until(str_addr as usize);
                    debug!("{:#x}: zstring \"{}\"", str_addr, string.orig);
                    let hexstrs: Vec<String> = string.chars.iter().map(|b| format!("{:02X}", b)).collect();
                    trace!("{:#x}: {}", str_addr, hexstrs.connect(" "));
                    self.data.append_bytes(&string.chars);
                    self.data.write_u16((str_addr/8) as u16, string.from_addr as usize);  // packed addr
                    str_addr
                } else {
                    string.written_addr
                };
                string.written_addr = n_str_addr;
                prev_strings.push((string.clone(), n_str_addr));
            }
        }
    }

    /// adds jump to write the jump-addresses after reading all commands
    pub fn add_jump(&mut self, name: String, jump_type: JumpType) {
        let from_addr: u32 = self.data.bytes.len() as u32;
        let jump: Zjump = Zjump{ from_addr: from_addr, name: name, jump_type: jump_type};
        self.jumps.push(jump);

        // spacer for the adress where the to-jump-label will be written
        self.data.write_u16(0x0000, from_addr as usize);
    }

    /// adds label to the labels-vector. we need them later
    fn add_label(&mut self, name: String, to_addr: u32) {
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
            &ZOP::Add{ref operand1, ref operand2, ref save_variable} => op::op_add(operand1, operand2, save_variable),
            &ZOP::Sub{ref operand1, ref operand2, ref save_variable} => op::op_sub(operand1, operand2, save_variable),
            &ZOP::Mul{ref operand1, ref operand2, ref save_variable} => op::op_mul(operand1, operand2, save_variable),
            &ZOP::Div{ref operand1, ref operand2, ref save_variable} => op::op_div(operand1, operand2, save_variable),
            &ZOP::Or{ref operand1, ref operand2, ref save_variable} => op::op_or(operand1, operand2, save_variable),
            &ZOP::And{ref operand1, ref operand2, ref save_variable} => op::op_and(operand1, operand2, save_variable),
            &ZOP::Mod{ref operand1, ref operand2, ref save_variable} => op::op_mod(operand1, operand2, save_variable),
            &ZOP::StoreVariable{ref variable, ref value} => op::op_store_var(variable, value),
            &ZOP::Ret{value} => op::op_ret(value),
            &ZOP::PrintAddr{ref address} => op::op_print_addr(address),
            &ZOP::PrintPaddr{ref address} => op::op_print_paddr(address),
            &ZOP::SetColor{foreground, background} => op::op_set_color(foreground, background),
            &ZOP::SetColorVar{foreground, background} => op::op_set_color_var(foreground, background),
            &ZOP::Random{ref range, ref variable} => op::op_random(range, variable),
            &ZOP::PrintNumVar{ref variable} => op::op_print_num_var(variable),
            &ZOP::SetTextStyle{bold, reverse, monospace, italic} => op::op_set_text_style(bold, reverse, monospace, italic),
            &ZOP::ReadChar{local_var_id} => op::op_read_char(local_var_id),
            &ZOP::LoadW{ref array_address, ref index, ref variable} => op::op_loadw(array_address, index, variable),
            &ZOP::StoreW{ref array_address, ref index, ref variable} => op::op_storew(array_address, index, variable),
            &ZOP::Call1NVar{variable} => op::op_call_1n_var(variable),
            &ZOP::EraseWindow{value} => op::op_erase_window(value),

            _ => Vec::new()
        };
        self.data.append_bytes(&bytes);
        match instr {
            &ZOP::PrintUnicode{c} => self.op_print_unicode_char(c),
            &ZOP::PrintUnicodeVar{ref var} => self.op_print_unicode_var(var),
            &ZOP::PrintUnicodeStr{ref address} => self.op_print_unicode_str(address),
            &ZOP::Print{ref text} => self.op_print(text),
            &ZOP::PrintOps{ref text} => self.gen_print_ops(text),
            &ZOP::Routine{ref name, count_variables} => self.routine(name, count_variables),
            &ZOP::Label{ref name} => self.label(name),
            &ZOP::Jump{ref jump_to_label} => self.op_jump(jump_to_label),
            &ZOP::ReadCharTimer{local_var_id, timer, ref routine} => self.op_read_char_timer(local_var_id, timer, routine),
            &ZOP::JL{ref operand1, ref operand2, ref jump_to_label} => self.op_jl(operand1, operand2, jump_to_label),
            &ZOP::JLE{ref operand1, ref operand2, ref jump_to_label} => self.op_jle(operand1, operand2, jump_to_label),
            &ZOP::JG{ref operand1, ref operand2, ref jump_to_label} => self.op_jg(operand1, operand2, jump_to_label),
            &ZOP::JGE{ref operand1, ref operand2, ref jump_to_label} => self.op_jge(operand1, operand2, jump_to_label),
            &ZOP::JE{ref operand1, ref operand2, ref jump_to_label} => self.op_je(operand1, operand2, jump_to_label),
            &ZOP::Call2NWithAddress{ref jump_to_label, ref address} => self.op_call_2n_with_address(jump_to_label, address),
            &ZOP::Call2NWithArg{ref jump_to_label, ref arg} => self.op_call_2n_with_arg(jump_to_label, arg),
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
                    // no space in table, so plain utf16 is the answer
                    trace!("Unicode char '{:?}' is not in unicode_table", character.to_string());
                    self.gen_write_out_zstring(current_text.to_string());  // write out previous buffer
                    current_text.clear();
                    current_utf16.push(character);
                }
            }
        }

        self.gen_write_out_unicode(current_utf16);  // write out utf16 string
        self.gen_write_out_zstring(current_text);  // order does not matter
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
                self.emit(vec![ZOP::Call2NWithArg{jump_to_label: "print_unicode".to_string(), arg: Operand::new_large_const(0)}]);
                self.strings.push(Zstring{chars: utf16bytes, orig: current_utf16.to_string(), from_addr: (self.data.len()-2) as u32, unicode: true, written_addr: 0});
            } else {
                self.emit(vec![ZOP::PrintUnicode{c: current_utf16.chars().nth(0).unwrap() as u16}]);
            }
        }
    }

    fn gen_write_out_zstring(&mut self, current_text: String) {
        if current_text.len() > 0 {
            if current_text.len() > 3 {  // write string to high mem
                self.gen_high_mem_zprint(&current_text[..]);
            } else {  // print in place
                self.emit(vec![ZOP::Print{text: current_text}]);
            }
        }
    }

    fn gen_high_mem_zprint(&mut self, text: &str) {
        self.emit(vec![ZOP::PrintPaddr{address: Operand::new_large_const(0)}]);  // dummy addr
        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        ztext::encode(&mut text_bytes, text, &self.unicode_table);
        self.strings.push(
            Zstring{chars: text_bytes.bytes, orig: text.to_string(), from_addr: (self.data.len()-2) as u32, unicode: false, written_addr: 0});
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
        let index: u32 = routine_address(self.data.bytes.len() as u32);
        
        assert!(count_variables <= 15, "only 15 local variables are allowed");
        assert!(index % 8 == 0, "adress of a routine must start at address % 8 == 0");

        self.add_label(name.to_string(), index);
        self.data.write_byte(count_variables, index as usize);
    }

    /// command to create a label
    pub fn label(&mut self, name: &str) {
        let index: usize = self.data.bytes.len();
        self.add_label(name.to_string(), index as u32);
    }

    // ================================
    // zcode routines

    /// routine to add the address of a passage-link
    pub fn routine_add_link(&mut self) {
        let save_at_addr: u16 = 1 + self.object_addr;
        self.emit(vec![
            ZOP::Routine{name: "system_add_link".to_string(), count_variables: 1},
            // saves routine-argument to array
            ZOP::StoreW{array_address: Operand::new_large_const(save_at_addr as i16), index: Variable::new(16), variable: Variable::new(1)},

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
        let save_at_addr: u16 = 1 + self.object_addr;
        self.emit(vec![
            ZOP::Routine{name: "system_check_links".to_string(), count_variables: 2},

            // jumps to the end, if this passage was called as <<display>>
            ZOP::JE{operand1: Operand::new_var(17), operand2: Operand::new_const(0x01), jump_to_label: "system_check_links_end_ret".to_string()},

            // jumps to the end, if there a no links
            ZOP::JE{operand1: Operand::new_var(16), operand2: Operand::new_const(0x00), jump_to_label: "system_check_links_end_quit".to_string()},
            ZOP::Print{text: "--------------------".to_string()},
            ZOP::Newline,
            ZOP::Print{text: "press a key... ".to_string()},
            ZOP::Newline,

            ZOP::Label{name: "system_check_links_loop".to_string()},
            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(129), jump_to_label: "system_check_links_jmp".to_string()},
            ZOP::Jump{jump_to_label: "system_check_links_after".to_string()},
            ZOP::Label{name: "system_check_links_jmp".to_string()},
            ZOP::Call1N{jump_to_label: "system_check_more".to_string()},

            ZOP::Label{name: "system_check_links_after".to_string()},

            ZOP::Sub{operand1: Operand::new_var(0x01), operand2: Operand::new_const(48), save_variable: Variable::new(0x01)},

            // check if the link in 0x01 exist, if not
            // => "wrong key => jump before key-detection
            ZOP::JL{operand1: Operand::new_var(16), operand2: Operand::new_var(0x01), jump_to_label: "system_check_links_loop".to_string()},

            // check if the key-48 is < 0, if it is => jump before key-detection
            ZOP::StoreVariable{variable: Variable::new(0x02), value: Operand::new_const(1)},
            ZOP::JL{operand1: Operand::new_var(0x01), operand2: Operand::new_var(0x02), jump_to_label: "system_check_links_loop".to_string()},
            ZOP::Dec{variable: 0x01},

            // loads the address of the link from the array
            ZOP::LoadW{array_address: Operand::new_large_const(save_at_addr as i16), index: Variable::new(1), variable: Variable::new(2)},

            // no more links exist
            ZOP::StoreVariable{variable: Variable::new(16), value: Operand::new_const(0)},
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
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(129), jump_to_label: "system_check_more_ko_1".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_1".to_string()},
        
            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(130), jump_to_label: "system_check_more_ko_2".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_2".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(130), jump_to_label: "system_check_more_ko_3".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_3".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(131), jump_to_label: "system_check_more_ko_4".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_4".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(132), jump_to_label: "system_check_more_ko_5".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_5".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(131), jump_to_label: "system_check_more_ko_6".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_6".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(132), jump_to_label: "system_check_more_ko_7".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_7".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(98), jump_to_label: "system_check_more_ko_8".to_string()},
            ZOP::Ret{value: 0},
            ZOP::Label{name: "system_check_more_ko_8".to_string()},

            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(97), jump_to_label: "system_check_more_ko_9".to_string()},
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

            ZOP::StoreVariable{variable: Variable::new(1), value: Operand::new_const(20)},
            ZOP::Label{name: "system_check_more_loop".to_string()},
            ZOP::Random{range: Operand::new_const(8), variable: Variable::new(4)},
            ZOP::Random{range: Operand::new_const(100), variable: Variable::new(5)},
            ZOP::Add{operand1: Operand::new_var(5), operand2: Operand::new_const(10), save_variable: Variable::new(5)},
            ZOP::Inc{variable: 4},
            ZOP::SetColorVar{foreground: 4, background: 4},
            ZOP::Print{text: "aa".to_string()},
            ZOP::Inc{variable: 2},

            ZOP::JL{operand1: Operand::new_var(2), operand2: Operand::new_var(1), jump_to_label: "system_check_more_loop".to_string()},
            ZOP::Newline,
            ZOP::Inc{variable: 3},
            ZOP::StoreVariable{variable: Variable::new(2), value: Operand::new_const(0)},
            ZOP::JL{operand1: Operand::new_var(3), operand2: Operand::new_var(1), jump_to_label: "system_check_more_loop".to_string()},
            ZOP::Ret{value: 0}
        ]);
    }

    /// print UTF-16 string from addr
    /// expects an address as argument where first the length (as u16)
    /// of the string is written (number of u16 chars), followed by the string to print.
    /// only works if the string is within the address space up to 0xffff
    pub fn routine_print_unicode(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "print_unicode".to_string(), count_variables: 4},
            // DEBUG    ZOP::Print{text: "pos:".to_string()}, ZOP::PrintNumVar{variable: 0x01},
            // addr as arg1 in 0x01, copy length to 0x02
            ZOP::LoadW{array_address: Operand::new_var(1), index: Variable::new(4), variable: Variable::new(2)},  // index at var:4 is 0
            // DEBUG    ZOP::Print{text: "len:".to_string()}, ZOP::PrintNumVar{variable: 0x02},
            ZOP::Add{operand1: Operand::new_var(2), operand2: Operand::new_var(2), save_variable: Variable::new(2)}, // double length
            ZOP::Add{operand1: Operand::new_var(1), operand2: Operand::new_var(2), save_variable: Variable::new(2)}, // add 'offset' addr to length,
            // so it marks the position after the last char after we increase it by 2 again
            ZOP::Add{operand1: Operand::new_var(2), operand2: Operand::new_large_const(2i16), save_variable: Variable::new(2)}, // point after last char
            ZOP::Add{operand1: Operand::new_var(1), operand2: Operand::new_large_const(2i16), save_variable: Variable::new(1)}, // point to first char
            ZOP::Label{name: "inter_char".to_string()},
            // DEBUG    ZOP::Print{text: "pos:".to_string()}, ZOP::PrintNumVar{variable: 0x01},
            // load u16 char to 0x3
            ZOP::LoadW{array_address: Operand::new_var(1), index: Variable::new(4), variable: Variable::new(3)},  // index at var:4 is 0
            // DEBUG    ZOP::Print{text: "code:".to_string()}, ZOP::PrintNumVar{variable: 0x03},
            ZOP::PrintUnicodeVar{var: Variable::new(3)},
            ZOP::Add{operand1: Operand::new_var(1), operand2: Operand::new_large_const(2i16), save_variable: Variable::new(1)}, // point to next char
            ZOP::JL{operand1: Operand::new_var(1), operand2: Operand::new_var(2), jump_to_label: "inter_char".to_string()},
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

    /// calls a routine with one argument an throws result away
    /// call_2n is 2OP
    pub fn op_call_2n_with_arg(&mut self, jump_to_label: &str, arg: &Operand) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, op::arg_type(&arg)];
        self.op_2(0x1a, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        op::write_argument(arg, &mut self.data.bytes);  // just one argument
    }

    /// jumps to a label if the value of operand1 is equal to operand2
    pub fn op_je(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![op::arg_type(operand1), op::arg_type(operand2)];
        self.op_2(0x01, args);
        
        op::write_argument(operand1, &mut self.data.bytes);
        op::write_argument(operand2, &mut self.data.bytes);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::Branch);
    }

    /// jumps to a label if the value of operand1 is lower than operand2 (compared as i16)
    pub fn op_jl(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![op::arg_type(operand1), op::arg_type(operand2)];
        self.op_2(0x02, args);

        op::write_argument(operand1, &mut self.data.bytes);
        op::write_argument(operand2, &mut self.data.bytes);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::Branch);
    }

    /// jumps to a label if the value of operand1 is lower than or equal operand2 (compared as i16)
    pub fn op_jle(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {
        self.emit(vec![
            ZOP::JL{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()},
            ZOP::JE{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()}
            ]);
    }

    /// jumps to a label if the value of operand1 is greater than or equal operand2 (compared as i16)
    pub fn op_jge(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {
        self.emit(vec![
            ZOP::JG{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()},
            ZOP::JE{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()}
            ]);
    }

    /// jumps to a label if the value of operand1 is greater than operand2
    pub fn op_jg(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![op::arg_type(operand1), op::arg_type(operand2)];
        self.op_2(0x03, args);

        op::write_argument(operand1, &mut self.data.bytes);
        op::write_argument(operand2, &mut self.data.bytes);

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
    pub fn op_print_unicode_var(&mut self, variable: &Variable) {

        self.op_1(0xbe, ArgType::SmallConst);
        self.data.append_byte(0x0b);
        // 0x00 means LargeConst, then 0x03 means omitted, 0x02 means Variable, 0x01 means SmallConst
        let byte = 0x02 << 6 | 0x03 << 4 | 0x03 << 2 | 0x03 << 0;
        self.data.append_byte(byte);
        self.data.append_byte(variable.id);
    }

    pub fn op_print_unicode_str(&mut self, address: &Operand) {
        self.emit(vec![ZOP::Call2NWithArg{jump_to_label: "print_unicode".to_string(), arg: address.clone()}]);
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

fn align_address(address: u32, align: u32) -> u32 {
    address + (align - (address % align)) % align
}

/// returns the routine address, should be adress % 8 == 0 (becouse its an packed address)
fn routine_address(address: u32) -> u32 {
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

#[test]
fn test_op_inc() {
    assert_eq!(op::op_inc(1),vec![0x95,0x01]);
}

#[test]
fn test_op_dec() {
    assert_eq!(op::op_dec(1),vec![0x96,0x01]);
}

#[test]
fn test_op_newline() {
    assert_eq!(op::op_newline(),vec![0xbb]);
}

#[test]
fn test_op_quit() {
    assert_eq!(op::quit(),vec![0xba]);
}

#[test]
fn test_op_add() {
    assert_eq!(op::op_add(&Operand::new_var(1),&Operand::new_var(2),&Variable::new(3)),vec![0x74,0x01,0x02,0x03]);
}
#[test]
fn test_op_sub() {
    assert_eq!(op::op_sub(&Operand::new_var(1),&Operand::new_var(2),&Variable::new(3)),vec![0x75,0x01,0x02,0x03]);
}

#[test]
fn test_op_mul() {
    assert_eq!(op::op_mul(&Operand::new_var(1),&Operand::new_var(2),&Variable::new(3)),vec![0x76,0x01,0x02,0x03]);
}

#[test]
fn test_op_div() {
    assert_eq!(op::op_div(&Operand::new_var(1),&Operand::new_var(2),&Variable::new(3)),vec![0x77,0x01,0x02,0x03]);
}

#[test]
fn test_op_mod() {
    assert_eq!(op::op_mod(&Operand::new_var(1),&Operand::new_var(2),&Variable::new(3)),vec![0x78,0x01,0x02,0x03]);
}

#[test]
fn test_op_and() {
    assert_eq!(op::op_and(&Operand::new_var(1),&Operand::new_var(2),&Variable::new(3)),vec![0x69,0x01,0x02,0x03]);
}

#[test]
fn test_op_or() {
    assert_eq!(op::op_or(&Operand::new_var(1),&Operand::new_var(2),&Variable::new(3)),vec![0x68,0x01,0x02,0x03]);
}

#[test]
fn test_op_set_color() {
    assert_eq!(op::op_set_color(0x15,0x20),vec![0x1B,0x15,0x20]);
}

#[test]
fn test_op_set_color_var() {
    assert_eq!(op::op_set_color_var(0x01,0x02),vec![0x7B,0x01,0x02]);
}

#[test]
fn test_op_push_u16() {
    assert_eq!(op::op_push_u16(0x0101),vec![0xE8,0x3F,0x01,0x01]);
}

#[test]
fn test_op_pull() {
    assert_eq!(op::op_pull(0x01),vec![0xE9,0x7F,0x01]);
}

#[test]
fn test_op_random() {
    assert_eq!(op::op_random(&Operand::new_var(10),&Variable::new(3)),vec![0xE7,0xBF,0x0a,0x03]);
}

#[test]
fn test_op_print_num_var() {
    assert_eq!(op::op_print_num_var(&Variable::new(3)),vec![0xE6,0xBF,0x03]);
}

#[test]
fn test_op_set_text_style() {
    assert_eq!(op::op_set_text_style(true,true,true,true),vec![0xF1,0x7F,0x0F]);
    assert_eq!(op::op_set_text_style(true,false,false,false),vec![0xF1,0x7F,0x02]);
    assert_eq!(op::op_set_text_style(false,true,false,false),vec![0xF1,0x7F,0x01]);
    assert_eq!(op::op_set_text_style(false,false,true,false),vec![0xF1,0x7F,0x08]);
    assert_eq!(op::op_set_text_style(false,false,false,true),vec![0xF1,0x7F,0x04]);
    assert_eq!(op::op_set_text_style(false,false,false,false),vec![0xF1,0x7F,0x00]);
}

#[test]
fn test_op_read_char() {
    assert_eq!(op::op_read_char(0x01),vec![0xF6,0x7F,0x00,0x01]);
}

#[test]
fn test_op_loadw() {
    assert_eq!(op::op_loadw(&Operand::new_var(1),&Variable::new(2),&Variable::new(3)),vec![0x6F,0x01,0x02,0x03]);
}

#[test]
fn test_op_storew() {
    assert_eq!(op::op_storew(&Operand::new_var(1),&Variable::new(2),&Variable::new(3)),vec![0xE1,0xAB,0x01,0x02,0x03]);
}

#[test]
fn test_op_erase_window() {
    assert_eq!(op::op_erase_window(0x01),vec![0xED,0x3F,0x00,0x01]);
}

#[test]
fn test_op_call_1n_var() {
    assert_eq!(op::op_call_1n_var(0x01),vec![0xAF,0x01]);
}

#[test]
fn test_op_print_paddr() {
    assert_eq!(op::op_print_paddr(&Operand::new_var(10)),vec![0xAD,0x0a]);
}

#[test]
fn test_op_print_addr() {
    assert_eq!(op::op_print_addr(&Operand::new_var(10)),vec![0xA7,0x0a]);
}

#[test]
fn test_op_ret() {
    assert_eq!(op::op_ret(0x0101),vec![0x8B,0x01,0x01]);
}

#[test]
fn test_op_store_var() {
    assert_eq!(op::op_store_var(&Variable::new(2),&Operand::new_var(10)),vec![0x2d,0x02,0x0a]);
}

#[test]
fn test_encode_variable_arguments() {
    assert_eq!(op::encode_variable_arguments(vec![ArgType::Variable]),0x80);
    assert_eq!(op::encode_variable_arguments(vec![ArgType::SmallConst]),0x40);
    assert_eq!(op::encode_variable_arguments(vec![ArgType::LargeConst]),0x00);
    assert_eq!(op::encode_variable_arguments(vec![ArgType::Nothing]),0xc0);
    assert_eq!(op::encode_variable_arguments(vec![ArgType::Reference]),0x40);
}

#[test]
fn test_op_2() {
    assert_eq!(op::op_2(0x02,vec![ArgType::Variable]),vec![0x42]);
    assert_eq!(op::op_2(0x02,vec![ArgType::LargeConst]),vec![0xc2,0x0f]);
    assert_eq!(op::op_2(0x02,vec![ArgType::SmallConst]),vec![0x02]);
    assert_eq!(op::op_2(0x02,vec![ArgType::Reference]),vec![0x02]);
}

#[test]
fn test_op_1() {
    assert_eq!(op::op_1(0x02,ArgType::Variable),vec![0xa2]);
    assert_eq!(op::op_1(0x02,ArgType::LargeConst),vec![0x82]);
    assert_eq!(op::op_1(0x02,ArgType::SmallConst),vec![0x92]);
    assert_eq!(op::op_1(0x02,ArgType::Reference),vec![0x92]);
}

#[test]
fn test_op_var() {
    assert_eq!(op::op_var(0x02,vec![ArgType::Variable]),vec![0xe2,0x80]);
    assert_eq!(op::op_var(0x02,vec![ArgType::LargeConst]),vec![0xe2,0x00]);
    assert_eq!(op::op_var(0x02,vec![ArgType::SmallConst]),vec![0xe2,0x40]);
    assert_eq!(op::op_var(0x02,vec![ArgType::Reference]),vec![0xe2,0x40]);
}

#[test]
fn test_op_0() {
    assert_eq!(op::op_0(0x02),vec![0xb2]);
    assert_eq!(op::op_0(0x04),vec![0xb4]);
    assert_eq!(op::op_0(0x08),vec![0xb8]);
    assert_eq!(op::op_0(0x03),vec![0xb3]);
}




























