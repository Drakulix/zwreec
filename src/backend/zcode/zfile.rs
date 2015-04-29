//! The `zfile` module contains functionality to create a zcode file
//! 

pub use super::zbytes::Bytes;
pub use super::ztext;

pub enum JumpType {
    JUMP,
    BRANCH,
    ROUTINE
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
    pub name: String,
    //pub is_routine: bool
}


impl Zfile {

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

        // object tabelle name
        self.write_object_name("object", 770);

        // dictionary
        let tmp: [u8; 4] = [0x00, 0x06, 0x00, 0x00];
        self.data.write_bytes(&tmp, dictionary_addr as usize);
    }

    /// writes the alphabet to index
    fn write_alphabet(&mut self, index: usize) {
        // TODO: is it possible to do this with map?
        let mut alpha_tmp: [u8; 78] = [0; 78];
        for i in 0..ztext::ALPHA.len() {
            alpha_tmp[i] = ztext::ALPHA[i] as u8;
        }
        self.data.write_bytes(&alpha_tmp, index);
    }

    /// writes the object-name to an index
    fn write_object_name(&mut self, name: &str, index: usize) {
        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        let length: u16 = ztext::encode(&mut text_bytes, name);

        // length ob object name
        self.data.write_byte(length as u8, index);

        self.data.write_bytes(&text_bytes.bytes, index + 1);
    }

    /// saves the addresses of the labels to the positions of the jump-ops
    fn write_jumps(&mut self) {
        for jump in self.jumps.iter_mut() {
            for label in self.labels.iter_mut() {
                if label.name == jump.name {
                    match jump.jump_type {
                        JumpType::ROUTINE => {
                            self.data.write_u16(label.to_addr / 8, jump.from_addr as usize);
                        },
                        JumpType::BRANCH => {
                            let mut new_addr: i32 = label.to_addr as i32 - jump.from_addr as i32;
                            new_addr &= 0x3fff;
                            new_addr |= 0x8000;
                            self.data.write_u16(new_addr as u16, jump.from_addr as usize);
                        },
                        JumpType::JUMP => {
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

    /// adds label
    fn add_label(&mut self, name: String, to_addr: u16) {
        let label: Zlabel = Zlabel{ name: name, to_addr: to_addr };
        self.labels.push(label);
    }

    /// returns the routine address, should be adress % 8 == 0 (becouse its an packed address)
    fn routine_address(&self, adress: usize) -> usize {
        adress + adress % 8
    }


    // ================================
    // no op-commands

    /// start of an zcode programm
    /// fills everying < program_addr with zeros
    pub fn start(&mut self) {
        self.data.write_zero_until(self.program_addr as usize);
    }

    /// writes all stuff that couldn't written directly
    pub fn end(&mut self) {
        self.write_jumps();
    }

    /// creates an routine-start
    pub fn routine(&mut self, name: &str, count_variables: u8) {    
        let index: usize = self.routine_address(self.data.bytes.len());
        
        if count_variables != 0 {
            panic!("variables are not implemented until now");
        }
        if index % 8 != 0 {
            panic!("adress of a routine must start at address % 8 == 0");
        }

        self.add_label(name.to_string(), index as u16);
        self.data.write_byte(count_variables, index);
    }

    /// creates an label
    pub fn label(&mut self, name: &str) {
        let index: usize = self.data.bytes.len();
        self.add_label(name.to_string(), index as u16);
    }


    // ================================
    // specific ops

    /// print is 0OP
    pub fn op_print(&mut self, content: &str) {
        let index: usize = self.data.bytes.len();
        self.op_0_op(0x02);

        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        ztext::encode(&mut text_bytes, content);
        self.data.write_bytes(&text_bytes.bytes, index + 1);
    }

    /// quit is 0OP
    pub fn op_quit(&mut self) {
        self.op_0_op(0x0a);
    }

    /// call_1n is 1OP
    pub fn op_call_1n(&mut self, jump_to_label: &str) {
        self.op_1_op(0x0f);
        self.add_jump(jump_to_label.to_string(), JumpType::ROUTINE);
    }

    /// read_char is VAROP
    pub fn op_read_char(&mut self, local_var_id: u8) {
        self.op_var(0x16);

        // type of following arguments
        // first argument has value 0 to detect value from keyboard
        // => 0x01, becouse of constant < 255
        // "zero the other 3 arguments"
        let byte = 0x01 << 6 | 0x03 << 4 | 0x03 << 2 | 0x03 << 0;
        self.data.append_byte(byte);

        // write argument value
        self.data.append_byte(0x00);

        // write varible id
        self.data.append_byte(local_var_id);
    }

    pub fn op_jump(&mut self, jump_to_label: &str) {
        self.op_1_op(0x0c);
        self.add_jump(jump_to_label.to_string(), JumpType::JUMP);
    }

    /// is an 2OP, but with small constant and variable
    pub fn op_je(&mut self, local_var_id: u8, equal_to_const: u8, jump_to_label: &str) {

        // 0x01: variable; 0x00: constant; 0x01: je-opcode
        let op_coding = 0x01 << 6 | 0x00 << 5 | 0x01;
        self.data.append_byte(op_coding);
        
        // variable id
        self.data.append_byte(local_var_id);

        // const
        self.data.append_byte(equal_to_const);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::BRANCH);
    }
    

    // ================================
    // general ops

    fn op_0_op(&mut self, value: u8) {
        let byte = value | 0xb0;
        self.data.append_byte(byte);
    }
    
    fn op_1_op(&mut self, value: u8) {
        let byte = value | 0x80;
        self.data.append_byte(byte);
    }

    /// only one variable is supportet at the moment
    fn op_var(&mut self, value: u8) {
        let byte = value | 0xe0;
        self.data.append_byte(byte);
    }
}
