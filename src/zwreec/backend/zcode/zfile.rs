//! The `zfile` module contains functionality to create a zcode file
//! 

pub use super::zbytes::Bytes;
pub use super::ztext;

#[derive(Debug)]
pub enum ZOP {
  StoreString{variable: u8, string: String},
  PrintUnicode{c: char},
  Print{text: String},
  PrintNumVar{variable: u8},
  PrintOps{text: String},
  Call{jump_to_label: String},
  CallWithAddress{jump_to_label: String, address: String},
  CallVar{variable: u8},
  Routine{name: String, count_variables: u8},
  Label{name: String},
  Newline,
  SetTextStyle{bold: bool, reverse: bool, monospace: bool, italic: bool},
  StoreU16{variable: u8, value: u16},
  StoreU8{variable: u8, value: u8},
  StoreW{array_address: u16, index: u8, variable: u8},
  Inc{variable: u8},
  Ret{value: u8},
  JE{local_var_id: u8, equal_to_const: u8, jump_to_label: String},
  ReadChar{local_var_id: u8},
  Sub{variable1: u8, sub_const: u16, variable2: u8},
  JL{local_var_id: u8, local_var_id2: u8, jump_to_label: String},
  Jump{jump_to_label: String},
  Dec{variable: u8},
  LoadW{array_address: u16, index: u8, variable: u8},
  Quit,
}

enum JumpType {
    Jump,
    Branch,
    Routine
}

/// types of possible arguments
enum ArgType {
    LargeConst,
    SmallConst,
    Variable,
    Reference,
    Nothing
}

pub struct Zfile {
    pub data: Bytes,
    program_addr: u16,
    jumps: Vec<Zjump>,
    labels: Vec<Zlabel>
}

struct Zjump {
    pub from_addr: u16,
    pub name: String,
    pub jump_type: JumpType
}

struct Zlabel {
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
            program_addr: 0x800,
            jumps: Vec::new(),
            labels: Vec::new()
        }
    }

    /// creates the header of a zfile
    pub fn create_header(&mut self) {
        
        assert!(self.data.len() == 0, "create_header should run at the beginning of the op-codes");

        let alpha_addr: u16 = 0x40;
        let extension_addr: u16 = alpha_addr + 78;
        let global_addr: u16 = extension_addr as u16 + 4;
        let object_addr: u16 = global_addr + 480;
        let high_memory_addr: u16 = 0x308;
        let static_addr: u16 = 0x308;
        let dictionary_addr: u16 = 0x308;
        //let program_addr: u16 = 0x800;

        // version
        self.data.write_byte(8, 0x00);

        // flags
        self.data.write_byte(0, 0x01);

        // release version (0x02 und 0x03)
        self.data.write_u16(0, 0x02);

        // base of high memory (byte address) (0x04 and 0x05)
        self.data.write_u16(high_memory_addr, 0x04);

        // initial value of programm counter (0x06 and 0x07)
        self.data.write_u16(self.program_addr, 0x06);

        // location of dictionary (byte address) (0x08 and 0x09)
        self.data.write_u16(dictionary_addr, 0x08);

        // location of object table (byte address) (0x0a and 0x0b)
        self.data.write_u16(object_addr, 0x0a);

        // location of global variables table (byte address) (0x0c and 0x0d)
        self.data.write_u16(global_addr, 0x0c);

        // base of static memory (byte address) (0x0e and 0x0f)
        self.data.write_u16(static_addr, 0x0e);

        // alphabet address (bytes) - its 0x34 and 0x35, why not only 0x34?
        self.data.write_u16(alpha_addr, 0x34);

        // header extension table address (bytes) - its 0x36 and 0x37, why not only 0x36?
        self.data.write_u16(extension_addr, 0x36);

        // alphabet
        self.write_alphabet(alpha_addr as usize);

        // header extension table
        self.data.write_u16(3, extension_addr as usize); // Number of further words in table
        self.data.write_u16(0, extension_addr as usize + 1); // x-coordinate of mouse after a click
        self.data.write_u16(0, extension_addr as usize + 2); // y-coordinate of mouse after a click
        self.data.write_u16(0, extension_addr as usize + 3); // if != 0: unicode translation table address (optional)

        // global variables
        // ...

        // object table name
        self.write_object_name("object", 770);

        // dictionary
        let tmp: [u8; 4] = [0x00, 0x06, 0x00, 0x00];
        self.data.write_bytes(&tmp, dictionary_addr as usize);
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

    /// writes the object-name to an index
    /// # Examples
    /// '''ignore
    /// write_object_name(name: 3, index: 10)
    /// '''
    fn write_object_name(&mut self, name: &str, index: usize) {
        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        let length: u16 = ztext::encode(&mut text_bytes, name);

        // length ob object name
        self.data.write_byte(length as u8, index);

        self.data.write_bytes(&text_bytes.bytes, index + 1);
    }

    /// saves the addresses of the labels to the positions of the jump-ops
    /// goes through all jumps and labels, if they have the same name:
    ///  write the "where to jump"-adress of the label to the position of the jump
    /// TODO: Error handling for nonexistant labels
    fn write_jumps(&mut self) {
        for jump in self.jumps.iter_mut() {
            for label in self.labels.iter_mut() {
                if label.name == jump.name {
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
        }
    }

    /// adds jump to write the jump-addresses after reading all commands
    fn add_jump(&mut self, name: String, jump_type: JumpType) {
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

    pub fn emit(&mut self, code: Vec<ZOP>) {
        for instr in &code {
            match instr {
                &ZOP::PrintUnicode{c} => self.op_print_unicode_char(c),
                &ZOP::Print{ref text} => self.op_print(text),
                &ZOP::PrintNumVar{variable} => self.op_print_num_var(variable),
                &ZOP::PrintOps{ref text} => self.gen_print_ops(text),
                &ZOP::Call{ref jump_to_label} => self.op_call_1n(jump_to_label),
                &ZOP::Routine{ref name, count_variables} => self.routine(name, count_variables),
                &ZOP::Label{ref name} => self.label(name),
                &ZOP::Newline => self.op_newline(),
                &ZOP::SetTextStyle{bold, reverse, monospace, italic} => self.op_set_text_style(bold, reverse, monospace, italic),
                &ZOP::CallWithAddress{ref jump_to_label, ref address} => self.op_call_2n_with_address(jump_to_label, address),
                &ZOP::StoreU16{variable, value} => self.op_store_u16(variable, value),
                &ZOP::StoreU8{variable, value} => self.op_store_u8(variable, value),
                &ZOP::StoreW{array_address, index, variable} => self.op_storew(array_address, index, variable),
                &ZOP::Inc{variable} => self.op_inc(variable),
                &ZOP::Ret{value} => self.op_ret(value),
                &ZOP::JE{local_var_id, equal_to_const, ref jump_to_label} => self.op_je(local_var_id, equal_to_const, jump_to_label),
                &ZOP::ReadChar{local_var_id} => self.op_read_char(local_var_id),
                &ZOP::Sub{variable1, sub_const, variable2} => self.op_sub(variable1, sub_const, variable2),
                &ZOP::JL{local_var_id, local_var_id2, ref jump_to_label} => self.op_jl(local_var_id, local_var_id2, jump_to_label),
                &ZOP::Jump{ref jump_to_label} => self.op_jump(jump_to_label),
                &ZOP::Dec{variable} => self.op_dec(variable),
                &ZOP::LoadW{array_address, index, variable} => self.op_loadw(array_address, index, variable),
                &ZOP::CallVar{variable} => self.op_call_1n_var(variable),
                &ZOP::Quit => self.op_quit(),
                &ZOP::StoreString{variable, ref string} => self.op_store_string(variable, string),
            }
        }
    }

    /// generates normal print opcodes for ASCII characters and unicode print
    /// opcodes for unicode characters
    pub fn gen_print_ops(&mut self, text: &str) {
        let mut current_text: String = String::new();
        for character in text.chars() {
            if character as u32 <= 126 {
                // this is a non-unicode char
                current_text.push(character);
            } else {
                debug!("Printing unicode char {}", character.to_string());
                // unicode
                if current_text.len() > 0 {
                    self.op_print(&current_text[..]);
                    current_text.clear();
                }

                self.op_print_unicode_char(character);
            }
        }

        if current_text.len() > 0 {
            self.op_print(&current_text[..]);
        }
    }

    // ================================
    // no op-commands

    /// start of an zcode programm
    /// fills everying < program_addr with zeros
    /// should called as the first commend
    pub fn start(&mut self) {
        self.create_header();
        self.data.write_zero_until(self.program_addr as usize);
    }

    /// writes all stuff that couldn't written directly
    /// should be called as the last commend
    pub fn end(&mut self) {
        self.routine_check_links();
        self.routine_add_link();
        self.write_jumps();
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
            ZOP::StoreW{array_address: 0, index: 16, variable: 0x01},
            //self.op_loadw(0, 0x00, 0x02);

            // inc the count links
            ZOP::Inc{variable: 16},
            ZOP::Ret{value: 0}
        ]);
    }

    /// checks all stored links and make them choiceable
    /// with the keyboard
    pub fn routine_check_links(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "system_check_links".to_string(), count_variables: 1},
            // jumps to the end, if there a no links
            ZOP::JE{local_var_id: 16, equal_to_const: 0x00, jump_to_label: "system_check_links_end".to_string()},
            ZOP::Print{text: "press a key... ".to_string()},
            ZOP::Newline,
            ZOP::Label{name: "system_check_links_loop".to_string()},
            ZOP::ReadChar{local_var_id: 0x01},
            ZOP::Sub{variable1: 0x01, sub_const: 48, variable2: 0x01},
            // check if the link in 0x01 exist, if not
            // => "wrong key => jump before key-detection
            ZOP::JL{local_var_id: 16, local_var_id2: 0x01, jump_to_label: "system_check_links_loop".to_string()},
            ZOP::Dec{variable: 0x01},
            // loads the address of the link from the array
            ZOP::LoadW{array_address: 0, index: 0x01, variable: 0x02},
            // no more links exist
            ZOP::StoreU8{variable: 16, value: 0},
            ZOP::Newline,
            // jump to the new passage
            ZOP::CallVar{variable: 0x02},
            ZOP::Label{name: "system_check_links_end".to_string()},
            ZOP::Quit
        ]);
    }



    // ================================
    // specific ops

    /// print strings
    /// print is 0OP
    pub fn op_print(&mut self, content: &str) {
        let index: usize = self.data.bytes.len();
        self.op_0(0x02);

        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        ztext::encode(&mut text_bytes, content);
        self.data.write_bytes(&text_bytes.bytes, index + 1);
    }

    /// exits the program
    /// quit is 0OP
    pub fn op_quit(&mut self) {
        self.op_0(0x0a);
    }

    pub fn op_newline(&mut self) {
        self.op_0(0x0b);
    }

    /// calls a routine
    /// call_1n is 1OP
    pub fn op_call_1n(&mut self, jump_to_label: &str) {
        self.op_1(0x0f, ArgType::SmallConst);
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);
    }

    /// calls a routine (the address is stored in a variable)
    pub fn op_call_1n_var(&mut self, variable: u8) {
        self.op_1(0x0f, ArgType::Variable);
        //self.add_jump(jump_to_label.to_string(), JumpType::Routine);
        self.data.append_byte(variable);
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

    /// subtraktion
    /// variable2 = variable1 - sub_const
    pub fn op_sub(&mut self, variable1: u8, sub_const: u16, variable2: u8) {
        let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::LargeConst];
        self.op_2(0x15, args);
        
        self.data.append_byte(variable1);
        self.data.append_u16(sub_const);
        self.data.append_byte(variable2);
    }

    // saves an u8 to the variable
    pub fn op_store_u8(&mut self, variable: u8, value: u8) {
        let args: Vec<ArgType> = vec![ArgType::Reference, ArgType::SmallConst];
        self.op_2(0x0d, args);

        self.data.append_byte(variable);
        self.data.append_byte(value);
    }

    // saves an u16 to the variable
    pub fn op_store_u16(&mut self, variable: u8, value: u16) {
        let args: Vec<ArgType> = vec![ArgType::Reference, ArgType::LargeConst];
        self.op_2(0x0d, args);

        self.data.append_byte(variable);
        self.data.append_u16(value);
    }

    /// increments the value of the variable
    pub fn op_inc(&mut self, variable: u8) {
        self.op_1(0x05, ArgType::Reference);
        self.data.append_byte(variable);
    }

    /// decrements the value of the variable
    pub fn op_dec(&mut self, variable: u8) {
        self.op_1(0x06, ArgType::Reference);
        self.data.append_byte(variable);
    }

    /// returns a SmallConst
    pub fn op_ret(&mut self, value: u8) {
        self.op_1(0x0b, ArgType::SmallConst);
        self.data.append_byte(value);
    }

    /// pushs an u16 value (for example an address) on the stack
    pub fn op_push_u16(&mut self, value: u16) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
        self.op_var(0x08, args);
        self.data.append_u16(value);
    }

    /// pulls an value off the stack to an variable
    /// SmallConst becouse pull takes an reference to an variable
    pub fn op_pull(&mut self, variable: u8) {
        let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
        self.op_var(0x09, args);

        self.data.append_byte(variable);
    }

    /// prints the value of a variable (only ints a possibe)
    pub fn op_print_num_var(&mut self, variable: u8) {
        let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
        self.op_var(0x06, args);

        self.data.append_byte(variable);
    }

   /// sets the colors of the foreground (font) and background
    pub fn op_set_color(&mut self, foreground: u8, background: u8){
        let op_coding = 0x00 << 6 | 0x00 << 5 | 0x1B;
        self.data.append_byte(op_coding);
        self.data.append_byte(foreground);
        self.data.append_byte(background);
    }

    /// set the style of the text
    pub fn op_set_text_style(&mut self, bold: bool, reverse: bool, monospace: bool, italic: bool){
        let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
        self.op_var(0x11, args);

        let mut style_byte : u8;
        style_byte = 0x00;
        if bold {
            style_byte |=0x02
        }
         if reverse {
            style_byte |=0x01
        }
         if monospace {
            style_byte |=0x08
        }
         if italic {
            style_byte |=0x04
        }
        self.data.append_byte(style_byte);
    }

    /// reads keys from the keyboard and saves the asci-value in local_var_id
    /// read_char is VAROP
    pub fn op_read_char(&mut self, local_var_id: u8) {
        let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
        self.op_var(0x16, args);

        // write argument value
        self.data.append_byte(0x00);

        // write varible id
        self.data.append_byte(local_var_id);
    }

    /// loads a word from an array in a variable
    /// loadw is an 2op, BUT with 3 ops -.-
    pub fn op_loadw(&mut self, array_address: u16, index: u8, variable: u8) {

        self.op_2(0x0f, vec![ArgType::LargeConst, ArgType::Variable]);

        // array address
        self.data.append_u16(array_address);

        // array index
        self.data.append_byte(index);

        // variable
        self.data.append_byte(variable);
    }

    /// stores a value to an array
    /// stores the value of variable to the address in: array_address + 2*index
    pub fn op_storew(&mut self, array_address: u16, index: u8, variable: u8) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::Variable, ArgType::Variable, ArgType::Nothing];
        self.op_var(0x01, args);

        // array address
        self.data.append_u16(array_address);

        // array index
        self.data.append_byte(index);

        // value
        self.data.append_byte(variable);
    }

    pub fn op_store_string(&mut self, variable: u8, string: &str){
    	//TODO: fill me
    }

    /// jumps to a label
    pub fn op_jump(&mut self, jump_to_label: &str) {
        self.op_1(0x0c, ArgType::SmallConst);
        self.add_jump(jump_to_label.to_string(), JumpType::Jump);
    }

    /// jumps to a label if the value of local_var_id is equal to const
    /// is an 2OP, but with small constant and variable
    pub fn op_je(&mut self, local_var_id: u8, equal_to_const: u8, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::SmallConst];
        self.op_2(0x01, args);
        
        // variable id
        self.data.append_byte(local_var_id);

        // const
        self.data.append_byte(equal_to_const);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::Branch);
    }

    /// jumps to a label if the value of local_var_id is equal to local_var_id2
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

    /// prints an unicode char to the current stream
    pub fn op_print_unicode_char(&mut self, character: char){
        let value = if character as u32 > 0xFFFF { '?' as u16 } else { character as u16 };
        self.op_1(0xbe, ArgType::SmallConst);
        self.data.append_byte(0x0b);
        let byte = 0x00 << 6 | 0x03 << 4 | 0x03 << 2 | 0x03 << 0;
        self.data.append_byte(byte);
        self.data.append_u16(value);
    }

    // ================================
    // general ops

    /// op-codes with 0 operators
    fn op_0(&mut self, value: u8) {
        let byte = value | 0xb0;
        self.data.append_byte(byte);
    }
    
    /// op-codes with 1 operator
    fn op_1(&mut self, value: u8, arg_type: ArgType) {
        let mut byte: u8 = 0x80 | value;

         match arg_type {
            ArgType::Reference  => byte |= 0x01 << 4,
            ArgType::Variable   => byte |= 0x02 << 4,
            ArgType::SmallConst => byte |= 0x00 << 4,
            _                   => panic!("no possible 1OP")
        }

        self.data.append_byte(byte);
    }

    /// op-codes with 2 operators
    fn op_2(&mut self, value: u8, arg_types: Vec<ArgType>) {
        let mut byte: u8 = 0x00;
        let mut is_variable: bool = false;
        for (i, arg_type) in arg_types.iter().enumerate() {
            let shift: u8 = 6 - i as u8;
            match arg_type {
                &ArgType::SmallConst => byte |= 0x00 << shift,
                &ArgType::Variable   => byte |= 0x01 << shift,
                &ArgType::Reference  => byte |= 0x00 << shift,
                &ArgType::LargeConst => is_variable = true,
                _                    => panic!("no possible 2OP")
            }
        }

        if is_variable {
            let mut byte: u8 = 0xc0 | value;
            byte = byte | value;
            self.data.append_byte(byte);

            let mut byte2 = self.encode_variable_arguments(arg_types);
            byte2 = byte2 | 0xf;
            self.data.append_byte(byte2);
        } else {
            byte = byte | value;
            self.data.append_byte(byte);
        }
    }

    /// op-codes with variable operators (4 are possible)
    fn op_var(&mut self, value: u8, arg_types: Vec<ArgType>) {
        // opcode
        let byte = value | 0xe0;
        self.data.append_byte(byte);

        let byte2: u8 = self.encode_variable_arguments(arg_types);
        self.data.append_byte(byte2);
    }

    /// encodes the argtypes for variable some 2OPs and varOPs
    fn encode_variable_arguments(&mut self, arg_types: Vec<ArgType>) -> u8 {
        let mut byte: u8 = 0x00;
        for (i, arg_type) in arg_types.iter().enumerate() {
            let shift: u8 = 6 - 2 * i as u8;
            match arg_type {
                &ArgType::LargeConst => byte |= 0x00 << shift,
                &ArgType::SmallConst => byte |= 0x01 << shift,
                &ArgType::Variable   => byte |= 0x02 << shift,
                &ArgType::Nothing    => byte |= 0x03 << shift,
                &ArgType::Reference  => byte |= 0x01 << shift,
                //_                    => panic!("no possible varOP")
            }
        }

        byte
        
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

    zfile.write_jumps();
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
    zfile.op_var(0x00, args);
    assert_eq!(zfile.data.len(), 5);
}
