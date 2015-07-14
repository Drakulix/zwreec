//! The `zfile` module contains functionality to create a Z-Code file.
//!
//! This contains all code related to operand handling, Z-Code instruction generation (except for
//! most op-codes) and all of shared structs and code used by codegen and other modules.

pub use super::zbytes::Bytes;
pub use super::ztext;
pub use super::ee::routine_easteregg;
pub use super::op;
use config::Config;

/// A variable type.
#[derive(Clone, PartialEq, Debug)]
#[allow(dead_code)]
pub enum Type {
    /// This type is not valid
    None = 0,

    /// This is a boolean type
    Bool = 1,

    /// This is an integer
    Integer = 2,

    /// This is a string
    String = 3,
}

/// A variable.
#[derive(Debug,Clone)]
pub struct Variable {
    /// The identifier of the variable. Identifiers 15 or below are local variables.
    pub id: u8,

    /// The type of the variable
    pub vartype: Type
}

/// An integer constant.
#[derive(Debug,Clone)]
pub struct Constant {
    /// The value of the constant
    pub value: u8
}

/// A signed 16-bit integer constant.
#[derive(Debug,Clone)]
pub struct LargeConstant {
    /// The value of the constant
    pub value: i16
}

/// There are three Operands in Z-Code:
/// Variables, SmallConsts and LargeConsts.
///
/// The other Operands are for a better code-readability.
#[derive(Debug,Clone)]
pub enum Operand {
    /// A variable
    Var(Variable),

    /// A small constant
    Const(Constant),

    /// A large constant
    LargeConst(LargeConstant),

    /// A string reference
    ///
    /// This is internally stored as a large constant containing the string address
    StringRef(LargeConstant),

    /// A boolean constant (stored as a small integer constant)
    BoolConst(Constant),
}

impl Operand {
    /// Creates a new constant with the specified value.
    pub fn new_const(value: u8) -> Operand {
        Operand::Const(Constant { value: value })
    }

    /// Creates a new large constant.
    pub fn new_large_const(value: i16) -> Operand {
        Operand::LargeConst(LargeConstant { value: value })
    }

    /// Creates a new string reference.
    pub fn new_string_ref(value: i16) -> Operand {
        Operand::StringRef(LargeConstant { value: value })
    }

    /// Creates a new variable.
    pub fn new_var(id: u8) -> Operand {
        Operand::Var(Variable::new(id))
    }

    /// Creates a new string reference variable.
    pub fn new_var_string(id: u8) -> Operand {
        Operand::Var(Variable::new_string(id))
    }

    /// Creates a new bool reference variable
    pub fn new_var_bool(id: u8) -> Operand {
        Operand::Var(Variable::new_bool(id))
    }

    /// Returns the value of the underlying constant.
    ///
    /// # Panics
    /// Panics if the Operand is not a constant.
    pub fn const_value(&self) -> i16 {
        match self {
            &Operand::Const(ref constant) => constant.value as i16,
            &Operand::LargeConst(ref constant) => constant.value,
            &Operand::BoolConst(ref constant) => constant.value as i16,
            _ => panic!("Operand must be a constant!")
        }
    }

    /// Returns whether the Operand is a constant.
    pub fn is_const(&self) -> bool {
        match self {
            &Operand::Const(_) | &Operand::LargeConst(_) | &Operand::BoolConst(_) => true,
            _ => false
        }
    }
}

impl Variable {
    /// Returns a new integer variable.
    pub fn new(id: u8) -> Variable {
        Variable { id: id, vartype: Type::Integer }
    }

    /// Returns a new string variable.
    pub fn new_string(id: u8) -> Variable {
        Variable { id: id, vartype: Type::String }
    }

    /// Returns a new bool variable.
    pub fn new_bool(id: u8) -> Variable {
        Variable { id: id, vartype: Type::Bool }
    }

    /// Returns a new variable of the Type specified by `vartype`.
    pub fn new_type(id: u8, vartype: Type) -> Variable {
        Variable { id: id, vartype: vartype }
    }
}

/// ZOP: Z-Code op code and pseudo op-code representation.
///
/// These structs contain instructions to generate Z-Machine op-codes.
/// To generate the op-codes see `backend::zcode::zfile::Zfile::emit`.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum ZOP {
    /// Prints a unicode character.
    PrintUnicode{c: u16},

    /// Prints the unicode character in the specified variable.
    /// `var` contains a 16 bit variable that contains the character.
    PrintUnicodeVar{var: Variable},

    /// Prints the unicode string at the address specified.
    PrintUnicodeStr{address: Operand},

    /// Print a ZSCII character.
    PrintChar{var: Variable},

    /// Print a ZSCII string.
    Print{text: String},

    /// Print a numeric variable.
    PrintNumVar{variable: Variable},

    /// Print a variable.
    PrintVar{variable: Variable},

    /// Print the ZSCII string at the packed address `address`.
    PrintPaddr{address: Operand},

    /// Print the ZSCII string at the large address specified.
    PrintAddr{address: Operand},

    /// Generate print op-codes for the specified string.
    ///
    /// This generates ZSCII op-codes for the supported characters and prints unicode characters
    /// separately.
    PrintOps{text: String},

    /// 1OP: Call a routine with one argument.
    Call1N{jump_to_label: String},

    /// 2OP: Call a routine with one argument (at the specified label) and throws the result away.
    Call2NWithAddress{jump_to_label: String, address: String},

    /// 2OP: Call a routine with one argument and store the result in `arg`.
    Call2NWithArg{jump_to_label: String, arg: Operand},

    /// 1OP: Call a routine with one variable argument.
    Call1NVar{variable: u8},

    /// 2OP: Call a routine with one argument and store the return value in `result`.
    Call2S{jump_to_label: String, arg: Operand, result: Variable},

    /// VAROP: Call a routine with two arguments and throw result away.
    CallVNA2{jump_to_label: String, arg1: Operand, arg2: Operand},

    /// VAROP: Call a routine with three arguments and throw result away.
    CallVNA3{jump_to_label: String, arg1: Operand, arg2: Operand, arg3: Operand},

    /// VAROP: Call a routine with two arguments and store result in `result`.
    CallVSA2{jump_to_label: String, arg1: Operand, arg2: Operand, result: Variable},

    /// VAROP: Call a routine with three arguments and store result in `result`.
    CallVSA3{jump_to_label: String, arg1: Operand, arg2: Operand, arg3: Operand, result: Variable},

    /// VAROP with types-byte: Call a routine with five arguments and store the return value in `result`.
    CallVS2A5{jump_to_label: String, arg1: Operand, arg2: Operand, arg3: Operand, arg4: Operand, arg5: Operand, result: Variable},

    /// Declares a Z-Routine
    Routine{name: String, count_variables: u8},

    /// Declares a label.
    ///
    /// This is only used internally to reference different parts of the generated file.
    Label{name: String},

    /// Prints a newline.
    Newline,

    /// Sets the foreground and background color to constants specified.
    SetColor{foreground: u8, background: u8},

    /// Sets the foreground and background color to the variables with the IDs specified.
    SetColorVar{foreground: u8, background: u8},

    /// Set text style to `bold`, `reverse` (inverse colors), `monospace` and `italic`.
    SetTextStyle{bold: bool, reverse: bool, monospace: bool, italic: bool},

    /// Store the value in `value` to the variable.
    StoreVariable{variable: Variable, value: Operand},

    /// Store the word in `variable` at `array_address + index`.
    StoreW{array_address: Operand, index: Variable, variable: Variable},

    /// Store the byte in `variable` at `array_address + index`.
    StoreB{array_address: Operand, index: Variable, variable: Variable},

    /// Store the byte in `operand` at `array_address + index`.
    StoreBOperand{array_address: Operand, index: Operand, operand: Operand},

    /// Load the byte at `array_address + index` into `variable`.
    LoadBOperand{array_address: Operand, index: Operand, variable: Variable},

    /// Push a variable on the stack.
    PushVar{variable: Variable},

    /// Pull a variable from the stack.
    PullVar{variable: Variable},

    /// Increment a variable by 1: `variable += 1`.
    Inc{variable: u8},

    /// Decrement a variable by 1: `variable -= 1`.
    Dec{variable: u8},

    /// Return from the Z-Routine with the value in the specified Operand.
    Ret{value: Operand},

    /// Jump if `operand1 == operand2`.
    JE{operand1: Operand, operand2: Operand, jump_to_label: String},

    /// Jump if `operand1 != operand2`.
    JNE{operand1: Operand, operand2: Operand, jump_to_label: String},

    /// Jump if `operand1 < operand2`.
    JL{operand1: Operand, operand2: Operand, jump_to_label: String},

    /// Jump if `operand1 <= operand2`.
    JLE{operand1: Operand, operand2: Operand, jump_to_label: String},

    /// Jump if `operand1 > operand2`.
    JG{operand1: Operand, operand2: Operand, jump_to_label: String},

    /// Jump if `operand1 >= operand2`.
    JGE{operand1: Operand, operand2: Operand, jump_to_label: String},

    /// Store a random number between 1 and `range` in `variable`.
    Random{range: Operand, variable: Variable},

    /// Read a character from standard input in the variable.
    ReadChar{local_var_id: u8},

    /// Read a character from standard input in the variable or time out after `timer / 10` seconds elapsed.
    ReadCharTimer{local_var_id: u8, timer: u8, routine: String},

    /// Helper function to add two values according to their types.
    AddTypes{operand1: Operand, operand2: Operand, tmp1: Variable, tmp2: Variable, save_variable: Variable},

    /// Add two values: `save_variable = operand1 + operand2`.
    Add{operand1: Operand, operand2: Operand, save_variable: Variable},

    /// Subtract two values: `save_variable = operand1 - operand2`.
    Sub{operand1: Operand, operand2: Operand, save_variable: Variable},

    /// Multiply two values: `save_variable = operand1 * operand2`.
    Mul{operand1: Operand, operand2: Operand, save_variable: Variable},

    /// Divide two values: `save_variable = operand1 / operand2`.
    Div{operand1: Operand, operand2: Operand, save_variable: Variable},

    /// Modulo operation: `save_variable = operand1 % operand2`.
    Mod{operand1: Operand, operand2: Operand, save_variable: Variable},

    /// Bitwise OR: `save_variable = operand1 | operand2`.
    Or{operand1: Operand, operand2: Operand, save_variable: Variable},

    /// Bitwise AND: `save_variable = operand1 & operand2`.
    And{operand1: Operand, operand2: Operand, save_variable: Variable},

    /// Bitwise NOT: `result = ~operand`.
    Not{operand: Operand, result: Variable},

    /// Jump to a label.
    Jump{jump_to_label: String},

    /// Loads a word: `variable = array_address[index]`.
    LoadW{array_address: Operand, index: Variable, variable: Variable},

    /// Positions the cursor at the specified `line` and `column`.
    SetCursor{line: u8, col: u8},

    /// Positions the cursor at the `line` and `column` in the given Operands.
    SetCursorOperand{row: Operand, col: Operand},

    /// Update the cursor position variable with current data.
    UpdateCursorPos,

    /// Store the current cursor position at `store_addr`.
    GetCursor{store_addr: Operand},

    /// Erase the entire window with the specified id.
    EraseWindow{value: i8},

    /// Erase the current line starting from the cursor.
    EraseLine,

    /// Changes the variable type of the specified variable.
    SetVarType{variable: Variable, vartype: Type},

    /// Copies the variable type of `from` to `variable`.
    CopyVarType{variable: Variable, from: Operand},

    /// Stores the variable type of `variable` in `result`.
    GetVarType{variable: Variable, result: Variable},

    /// Quits the Z-Machine interpreter immediately.
    Quit,
}

/// Zcode has the jump-types:
///
/// jumps (to a label),
/// branches (to a label, from a compare-op like je, ...),
/// routine (to a routine-address)
#[derive(Debug, PartialEq, Clone)]
pub enum JumpType {
    /// Jump to a label (address)
    Jump,

    /// Conditionally jump to a label (with compare op-codes like JE)
    Branch,

    /// Call a routine
    Routine
}

/// Types of possible arguments.
pub enum ArgType {
    /// Large constant
    LargeConst,

    /// Small constant
    SmallConst,

    /// Variable
    Variable,

    /// Pointer to a string
    Reference,

    /// No argument
    Nothing
}

/// The definition of a Z-Code file.
pub struct Zfile {
    /// The output data
    pub data: Bytes,

    /// The unicode translation table
    unicode_table: Vec<u16>,

    /// A list of all jumps
    jumps: Vec<Zjump>,

    /// A list of all labels
    labels: Vec<Zlabel>,

    /// A list of all strings (used to find duplicate strings)
    strings: Vec<Zstring>,

    /// The beginning of executable code
    program_addr: u16,

    /// The address of the unicode translation table
    unicode_table_addr: u16,

    /// The address of the global variables
    global_addr: u16,

    /// Base of static memory
    static_addr: u16,

    /// Location of object table
    pub object_addr: u16,

    /// Location of the last write in static memory
    last_static_written: u16,

    /// Location of the type storage
    pub type_store: u16,

    /// Location of the cursor position
    pub cursor_pos: u16,

    /// Start of dynamic memory
    pub heap_start: u16,

    /// Flag to enable black font on white background
    pub bright_mode: bool,

    /// Force print_unicode op-code generation and omit unicode-translation table generation
    pub force_unicode: bool,

    /// Enable the easter-egg
    pub easter_egg: bool,

    /// Disable colours
    pub no_colours: bool,

    /// Disable unicode completely
    pub no_unicode: bool,
}

/// A jump.
#[derive(Debug, PartialEq, Clone)]
pub struct Zjump {
    /// The address the jump location should be stored at
    pub from_addr: u32,

    /// The label of the jump
    pub name: String,

    /// The type of jump
    pub jump_type: JumpType
}

/// A string.
#[derive(Debug, PartialEq, Clone)]
pub struct Zstring {
    /// the address where the string address should be stored at
    pub from_addr: u32,

    /// The character data
    /// Contains either ztext or [length: u16, utf16char:u16, …]
    pub chars: Vec<u8>,

    /// The original string
    pub orig: String,

    /// Contains whether the string is a unicode string or ZSCII
    pub unicode: bool,

    /// If the string data was already written to a location this is the address
    pub written_addr: u32,
}

/// A label.
#[derive(Debug, PartialEq, Clone)]
pub struct Zlabel {
    /// The address of the label
    pub to_addr: u32,

    /// The name of the label
    pub name: String
}

/// A formatting type.
///
/// zfile supports 4 formating possibilites: bold, mono, italic and inverted.
#[derive(Debug, Copy, Clone)]
pub struct FormattingState {
    /// Bold text
    pub bold: bool,

    /// Monospace text
    pub mono: bool,

    /// Italic text
    pub italic: bool,

    /// Inverted foreground and background colour
    pub inverted: bool
}


impl Zfile {
    /// Creates a new zfile with default options.
    pub fn new() -> Zfile {
        Zfile::new_with_options(false, false, false, false, false, false)
    }

    /// Creates a new zfile with the specified options.
    pub fn new_with_options(bright_mode: bool, force_unicode: bool, easter_egg: bool, no_colours: bool, half_memory: bool, no_unicode: bool) -> Zfile {
        Zfile {
            data: Bytes{bytes: Vec::new()},
            unicode_table: Vec::new(),
            jumps: Vec::new(),
            labels: Vec::new(),
            strings: Vec::new(),
            program_addr: if half_memory { 0x7918 } else { 0xfff8 },
            unicode_table_addr: 0,
            global_addr: 0,
            object_addr: 0,
            static_addr: 0,
            last_static_written: if half_memory { 0x4000 } else { 0x8000 },
            heap_start: 0x600,
            cursor_pos: 0x502,  // set by UpdateCursorPos
            type_store: 0x400,
            bright_mode: bright_mode,
            force_unicode: force_unicode,
            easter_egg: easter_egg,
            no_colours: no_colours,
            no_unicode: no_unicode,
        }
    }

    /// Creates a new zfile with the specified config.
    pub fn new_with_cfg(cfg: &Config) -> Zfile {
        Zfile::new_with_options(cfg.bright_mode, cfg.force_unicode, cfg.easter_egg, cfg.no_colours, cfg.half_memory, cfg.no_unicode)
    }

    /// Creates the header of a zfile.
    pub fn create_header(&mut self) {
        assert!(self.data.len() == 0, "create_header should run at the beginning of the op-codes");

        let alpha_addr: u16 = 0x40;
        let extension_addr: u16 = alpha_addr + 78;
        self.unicode_table_addr = extension_addr as u16 + 8;

        // 1 byte for the unicode count, 97 possible chars with 2 bytes
        self.global_addr = self.unicode_table_addr + 195;

        // 480 because there are 240 global 2-bytes variables
        self.object_addr = self.global_addr + 480;
        let high_memory_addr: u16 = self.program_addr;
        self.static_addr = self.last_static_written;
        let dictionary_addr: u16 = self.last_static_written;

        // version
        self.data.write_byte(8, 0x00);

        // flag1 (from right to left):
        // 0: colours available,
        // 1: picture,
        // 2: bold,
        // 3: italic,
        // 4: fixed
        self.data.write_byte(if self.no_colours { 0x1c } else { 0x1d } , 0x01);

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
        self.data.write_u16(self.static_addr, 0x0e);

        // alphabet address (bytes) - its 0x34 and 0x35, why not only 0x34?
        self.data.write_u16(alpha_addr, 0x34);

        // header extension table address (bytes) - its 0x36 and 0x37, why not only 0x36?
        self.data.write_u16(extension_addr, 0x36);

        // alphabet
        self.write_alphabet(alpha_addr as usize);

        // header extension table
        self.data.write_u16(3, extension_addr as usize);     // Number of further words in table
        self.data.write_u16(0, extension_addr as usize + 2); // x-coordinate of mouse after a click
        self.data.write_u16(0, extension_addr as usize + 4); // y-coordinate of mouse after a click
        self.data.write_u16(self.unicode_table_addr, extension_addr as usize + 6); // if != 0: unicode translation table address (optional)

        // global variables
        // ...
    }

    /// Writes the alphabet to index.
    fn write_alphabet(&mut self, index: usize) {
        // TODO: is it possible to do this with map?
        let mut alpha_tmp: [u8; 78] = [0; 78];
        for i in 0..ztext::ALPHABET.len() {
            alpha_tmp[i] = ztext::ALPHABET[i] as u8;
        }
        self.data.write_bytes(&alpha_tmp, index);
    }

    /// Writes the unicode translation table to the address unicode_table_addr.
    fn write_unicode_table(&mut self) {
        self.data.write_byte(self.unicode_table.len() as u8, self.unicode_table_addr as usize);

        for (i, character) in self.unicode_table.iter().enumerate() {
            self.data.write_u16(*character, self.unicode_table_addr as usize + 1 + 2*i);
        }

    }

    /// Saves the addresses of the labels to the positions of the jump-ops.
    ///
    /// This iterates through all jumps and labels and if they have the same name
    /// it writes the "where to jump"-adress of the label to the position of the jump.
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

    /// Saves the string to high memory.
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

    /// Saves the zstrings to high memory and writes the resulting address to the
    /// print_paddr arguments which referencing the string.
    fn write_strings(&mut self) {
        let mut prev_strings: Vec<(Zstring, u32)> = vec![];
        for string in self.strings.iter_mut() {
            // optimize to reuse strings if they are the same
            let mut string_found = false;
            for &(ref other, addr) in prev_strings.iter() {
                if other.unicode == string.unicode && other.orig == string.orig {
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

    /// Adds a jump to write the jump-addresses after reading all commands.
    pub fn add_jump(&mut self, name: String, jump_type: JumpType) {
        let from_addr: u32 = self.data.bytes.len() as u32;
        let jump: Zjump = Zjump{ from_addr: from_addr, name: name, jump_type: jump_type};
        self.jumps.push(jump);

        // spacer for the adress where the to-jump-label will be written
        self.data.write_u16(0x0000, from_addr as usize);
    }

    /// Adds a label to the labels-vector. we need them later.
    fn add_label(&mut self, name: String, to_addr: u32) {
        let label: Zlabel = Zlabel{ name: name, to_addr: to_addr };
        for other_label in self.labels.iter() {
            if other_label.name == label.name {
                panic!("label has to be unique, but \"{}\" isn't.", other_label.name);
            }
        }
        self.labels.push(label);
    }

    /// Write out the ZOP instructions to the data.
    pub fn emit(&mut self, code: Vec<ZOP>) {
        for instr in &code {
            let addr = self.data.bytes.len();
            debug!("{:#x}: {:?}", addr, instr);
            let (_, _, bytes) = self.write_zop(instr, false);
            let hexstrs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            trace!("{:#x}: {}", addr, hexstrs.connect(" "));
        }
    }

    /// Write opcodes to data array but also return written bytes for testing purposes as well as
    /// the resulting new labels and jumps.
    pub fn write_zop(&mut self, instr: &ZOP, return_new_jumps: bool) -> (Vec<Zlabel>, Vec<Zjump>, Vec<u8>){
        let beginning = self.data.bytes.len();
        let old_labels: Vec<Zlabel> = if return_new_jumps {
            self.labels.clone()
        } else {
            Vec::new()
        };

        let old_jumps: Vec<Zjump> = if return_new_jumps {
            self.jumps.clone()
        } else {
            Vec::new()
        };

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
            &ZOP::Not{ref operand, ref result} => op::op_not(operand, result),
            &ZOP::StoreVariable{ref variable, ref value} => op::op_store_var(variable, value),
            &ZOP::Ret{ref value} => op::op_ret(value),
            &ZOP::PrintAddr{ref address} => op::op_print_addr(address),
            &ZOP::PrintPaddr{ref address} => op::op_print_paddr(address),
            &ZOP::SetColor{foreground, background} => if self.no_colours { Vec::new() } else { op::op_set_color(foreground, background) },
            &ZOP::SetColorVar{foreground, background} => if self.no_colours { Vec::new() } else {  op::op_set_color_var(foreground, background) },
            &ZOP::Random{ref range, ref variable} => op::op_random(range, variable),
            &ZOP::PrintNumVar{ref variable} => op::op_print_num_var(variable),
            &ZOP::SetTextStyle{bold, reverse, monospace, italic} => if self.no_colours { Vec::new() } else { op::op_set_text_style(bold, reverse, monospace, italic) },
            &ZOP::ReadChar{local_var_id} => op::op_read_char(local_var_id),
            &ZOP::LoadW{ref array_address, ref index, ref variable} => op::op_loadw(array_address, index, variable),
            &ZOP::StoreW{ref array_address, ref index, ref variable} => op::op_storew(array_address, index, variable),
            &ZOP::StoreB{ref array_address, ref index, ref variable} => op::op_storeb(array_address, index, variable),
            &ZOP::StoreBOperand{ref array_address, ref index, ref operand} => op::op_storeboperand(array_address, index, operand),
            &ZOP::LoadBOperand{ref array_address, ref index, ref variable} => op::op_loadb(array_address, index, variable),
            &ZOP::Call1NVar{variable} => op::op_call_1n_var(variable),
            &ZOP::EraseWindow{value} => op::op_erase_window(value),
            &ZOP::EraseLine => op::op_erase_line(),
            &ZOP::SetCursor{line, col} => op::op_set_cursor(line, col),
            &ZOP::SetCursorOperand{ref row, ref col} => op::op_set_cursor_operand(row, col),
            &ZOP::PushVar{ref variable} => op::op_push_var(variable),
            &ZOP::PullVar{ref variable} => op::op_pull(variable.id.clone()),
            &ZOP::GetCursor{ref store_addr} => op::op_get_cursor(store_addr),

            _ => Vec::new()
        };
        self.data.append_bytes(&bytes);
        match instr {
            &ZOP::PrintUnicode{c} => self.op_print_unicode_char(c),
            &ZOP::PrintUnicodeVar{ref var} => if self.no_unicode == false { self.op_print_unicode_var(var) } else { self.op_call_2n_with_arg("print_char", &Operand::new_var(var.id.clone())) },
            &ZOP::PrintChar{ref var} => self.op_print_char(var),
            &ZOP::PrintUnicodeStr{ref address} => self.op_print_unicode_str(address),
            &ZOP::Print{ref text} => self.op_print(text),
            &ZOP::PrintOps{ref text} => self.gen_print_ops(text),
            &ZOP::PrintVar{ref variable} => self.print_var(variable),
            &ZOP::AddTypes{ref operand1, ref operand2, ref tmp1, ref tmp2, ref save_variable} => self.add_types(operand1, operand2, tmp1, tmp2, save_variable),
            &ZOP::Routine{ref name, count_variables} => self.routine(name, count_variables),
            &ZOP::Label{ref name} => self.label(name),
            &ZOP::Jump{ref jump_to_label} => self.op_jump(jump_to_label),
            &ZOP::ReadCharTimer{local_var_id, timer, ref routine} => self.op_read_char_timer(local_var_id, timer, routine),
            &ZOP::JL{ref operand1, ref operand2, ref jump_to_label} => self.op_jl(operand1, operand2, jump_to_label),
            &ZOP::JLE{ref operand1, ref operand2, ref jump_to_label} => self.op_jle(operand1, operand2, jump_to_label),
            &ZOP::JG{ref operand1, ref operand2, ref jump_to_label} => self.op_jg(operand1, operand2, jump_to_label),
            &ZOP::JGE{ref operand1, ref operand2, ref jump_to_label} => self.op_jge(operand1, operand2, jump_to_label),
            &ZOP::JE{ref operand1, ref operand2, ref jump_to_label} => self.op_je(operand1, operand2, jump_to_label),
            &ZOP::JNE{ref operand1, ref operand2, ref jump_to_label} => self.op_jne(operand1, operand2, jump_to_label),
            &ZOP::Call2NWithAddress{ref jump_to_label, ref address} => self.op_call_2n_with_address(jump_to_label, address),
            &ZOP::Call2NWithArg{ref jump_to_label, ref arg} => self.op_call_2n_with_arg(jump_to_label, arg),
            &ZOP::Call1N{ref jump_to_label} => self.op_call_1n(jump_to_label),
            &ZOP::Call2S{ref jump_to_label, ref arg, ref result} => self.op_call_2s(jump_to_label, arg, result),
            &ZOP::CallVNA2{ref jump_to_label, ref arg1, ref arg2} => self.op_call_vn_a2(jump_to_label, arg1, arg2),
            &ZOP::CallVNA3{ref jump_to_label, ref arg1, ref arg2, ref arg3} => self.op_call_vn_a3(jump_to_label, arg1, arg2, arg3),
            &ZOP::CallVSA2{ref jump_to_label, ref arg1, ref arg2, ref result} => self.op_call_vs_a2(jump_to_label, arg1, arg2, result),
            &ZOP::CallVSA3{ref jump_to_label, ref arg1, ref arg2, ref arg3, ref result} => self.op_call_vs_a3(jump_to_label, arg1, arg2, arg3, result),
            &ZOP::CallVS2A5{ref jump_to_label, ref arg1, ref arg2, ref arg3, ref arg4, ref arg5, ref result} => self.op_call_vs2_a5(jump_to_label, arg1, arg2, arg3, arg4, arg5, result),
            &ZOP::SetVarType{ref variable, ref vartype} => self.set_var_type(variable, vartype),
            &ZOP::CopyVarType{ref variable, ref from} => self.copy_var_type(variable, from),
            &ZOP::GetVarType{ref variable, ref result} => self.get_var_type(variable, result),
            &ZOP::UpdateCursorPos => self.update_cursor_pos(),
            _ => ()
        }
        let mut new_jumps: Vec<Zjump> = vec![];
        let mut new_labels: Vec<Zlabel> = vec![];

        if return_new_jumps {
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
        }

        (new_labels, new_jumps, self.data.bytes[beginning..self.data.bytes.len()].to_vec())
    }

    /// Generates normal print opcodes for ASCII characters and unicode print opcodes for unicode
    /// characters. Adds new characters to the unicode translation table if there is still space.
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
                if self.force_unicode == false && ztext::pos_in_unicode(character as u16, &self.unicode_table) != -1 {
                    self.gen_write_out_unicode(current_utf16.to_string());  // write out utf16 string
                    current_utf16.clear();
                    // unicode exists in table
                    current_text.push(character);
                } else if self.force_unicode == false && self.unicode_table.len() < 96 {
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

    /// Generates print_unicode opcodes for a given string.
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
                if let Some(temp) = current_utf16.chars().nth(0) {
                    self.emit(vec![ZOP::PrintUnicode{c: temp as u16}]);
                } else {
                    panic!{"No chars in current_utf16, can't print anything."}
                }
            }
        }
    }

    /// Writes a zstring to high memory or, if three bytes or smaller, directly after the
    /// instruction. Generates a print opcode at the current position.
    fn gen_write_out_zstring(&mut self, current_text: String) {
        if current_text.len() > 0 {
            if current_text.len() > 3 {  // write string to high mem
                self.gen_high_mem_zprint(&current_text[..]);
            } else {  // print in place
                self.emit(vec![ZOP::Print{text: current_text}]);
            }
        }
    }

    /// Writes a zstring to high memory and generates a print instruction.
    fn gen_high_mem_zprint(&mut self, text: &str) {
        self.emit(vec![ZOP::PrintPaddr{address: Operand::new_large_const(0)}]);  // dummy addr
        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        ztext::encode(&mut text_bytes, text, &self.unicode_table);
        self.strings.push(
            Zstring{
                chars: text_bytes.bytes,
                orig: text.to_string(),
                from_addr: (self.data.len()-2) as u32,
                unicode: false,
                written_addr: 0
            }
        );
    }

    // ================================
    // no op-commands

    /// Start of a zcode program.
    ///
    /// Fills everything < program_addr with zeros.
    ///
    /// # Caution
    /// This should be called as the first command.
    pub fn start(&mut self) {
        self.create_header();
        self.data.write_zero_until(self.program_addr as usize);

        let foreground: u8 = if self.bright_mode { 2 } else { 9 };
        let background: u8 = if self.bright_mode { 9 } else { 2 };

        // default theme and erase_window to fore the color
        self.emit(vec![
            ZOP::SetColor{foreground: foreground, background: background},
            ZOP::EraseWindow{value: -1},
            ZOP::Call1N{jump_to_label: "malloc_init".to_string()},
            ZOP::Call1N{jump_to_label: "Start".to_string()},
            ZOP::Label{name: "mainloop".to_string()},
            ZOP::Call1N{jump_to_label: "system_check_links".to_string()},
            ZOP::Jump{jump_to_label: "mainloop".to_string()},
        ]);
    }

    /// Writes all stuff that couldn't be written directly.
    ///
    /// # Caution
    /// This should be called as the last command.
    pub fn end(&mut self) {
        self.write_unicode_table();
        self.routine_check_links();
        self.routine_add_link();
        self.routine_check_more();
        self.routine_prompt();
        self.routine_print_unicode();
        self.routine_mem_free();
        self.routine_manual_free();
        self.routine_malloc_init();
        self.routine_strcpy();
        self.routine_strcmp();
        self.routine_malloc();
        self.routine_strcat();
        self.routine_itoa();
        self.routine_print_var();
        self.routine_print_char();
        self.routine_add_types();
        self.write_jumps();
        self.write_strings();
    }

    /// Command to create a Z-Routine.
    pub fn routine(&mut self, name: &str, count_variables: u8) {
        let index: u32 = routine_address(self.data.bytes.len() as u32);

        assert!(count_variables <= 15, "only 15 local variables are allowed");
        assert!(index % 8 == 0, "adress of a routine must start at address % 8 == 0");

        self.add_label(name.to_string(), index);
        self.data.write_byte(count_variables, index as usize);
    }

    /// Command to create a label.
    pub fn label(&mut self, name: &str) {
        let index: usize = self.data.bytes.len();
        self.add_label(name.to_string(), index as u32);
    }

    // ================================
    // zcode routines

    /// Routine to add the address of a passage-link.
    pub fn routine_add_link(&mut self) {
        let save_at_addr: u16 = 1 + self.object_addr;
        self.emit(vec![
            ZOP::Routine{name: "system_add_link".to_string(), count_variables: 1},
            // saves routine-argument to array
            ZOP::StoreW{array_address: Operand::new_large_const(save_at_addr as i16), index: Variable::new(16), variable: Variable::new(1)},

            // inc the count links
            ZOP::Inc{variable: 16},

            ZOP::Ret{value: Operand::new_const(0)}
        ]);
    }

    /// Exits the program immediately.
    ///
    /// quit is 0OP
    pub fn op_quit(&mut self) {
        self.op_0(0x0a);
    }


    /// Checks all stored links and make them selectable with the keyboard.
    ///
    /// The routine checks if there are < 10 links or more:
    ///
    /// if < 10: number keys 1-9 are supported, jumps immediately.
    ///
    /// if >=10: 99 links are supported, leading zeroes are not allowed.
    /// To jump to a link with a number smaller than 10 you have to press enter.
    pub fn routine_check_links(&mut self) {
        let save_at_addr: u16 = 1 + self.object_addr;
        self.emit(vec![
            ZOP::Routine{name: "system_check_links".to_string(), count_variables: 3},
            ZOP::Newline,

            // jumps to the end, if this passage was called as <<display>>
            ZOP::JE{operand1: Operand::new_var(17), operand2: Operand::new_const(0x01), jump_to_label: "system_check_links_end_ret".to_string()},

            // jumps to the end, if there a no links
            ZOP::JE{operand1: Operand::new_var(16), operand2: Operand::new_const(0x00), jump_to_label: "system_check_links_end_quit".to_string()},
            ZOP::SetTextStyle{bold: false, reverse: false, monospace: true, italic: false},
            ZOP::Print{text: "---------------------------------------".to_string()},
            ZOP::Newline,
            ZOP::Print{text: "Please press a number to select a link (end with Q):".to_string()},
            ZOP::Newline,

            // check if there are more than 9 links
            ZOP::JG{operand1: Operand::new_var(16), operand2: Operand::new_const(9), jump_to_label: "system_check_links_more_than_9".to_string()},

            // detect keys for <9 links
            ZOP::Label{name: "system_check_links_loop".to_string()},
            ZOP::ReadChar{local_var_id: 0x01},
            // Quit programme on Q
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(81), jump_to_label: "system_check_links_end_quit".to_string()},
            // check for the start of the konami code
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(129), jump_to_label: "system_check_links_jmp".to_string()},
            ZOP::Jump{jump_to_label: "system_check_links_after".to_string()},
            ZOP::Label{name: "system_check_links_jmp".to_string()},
            ZOP::Call1N{jump_to_label: "system_check_more".to_string()},

            ZOP::Label{name: "system_check_links_after".to_string()},
            ZOP::Sub{operand1: Operand::new_var(1), operand2: Operand::new_const(48), save_variable: Variable::new(1)},
            // check if the the detected key is > numbers of links
            // => "wrong key => jump before key-detection
            ZOP::JG{operand1: Operand::new_var(1), operand2: Operand::new_var(16), jump_to_label: "system_check_links_loop".to_string()},
            // check if key < 1, 0 is not valid
            ZOP::JL{operand1: Operand::new_var(1), operand2: Operand::new_const(1), jump_to_label: "system_check_links_loop".to_string()},
            // jump over the >9 links test
            // stores the index in 3
            ZOP::StoreVariable{variable: Variable::new(3), value: Operand::new_var(1)},
            ZOP::Jump{jump_to_label: "system_check_links_load_link_address".to_string()},

            // detect keys for >9 links
            ZOP::Label{name: "system_check_links_more_than_9".to_string()},
            // detect frst position
            ZOP::ReadChar{local_var_id: 1},
            // Quit programme on Q
            ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(81), jump_to_label: "system_check_links_end_quit".to_string()},
            ZOP::Sub{operand1: Operand::new_var(1), operand2: Operand::new_const(48), save_variable: Variable::new(1)},
            ZOP::PrintNumVar{variable: Variable::new(1)},

            // check if the the detected key is > 9
            ZOP::JG{operand1: Operand::new_var(1), operand2: Operand::new_const(9), jump_to_label: "system_check_links_error".to_string()},
            // check if key < 1, 0 is not valid
            ZOP::JL{operand1: Operand::new_var(1), operand2: Operand::new_const(1), jump_to_label: "system_check_links_error".to_string()},
            // stores the index in 3
            ZOP::StoreVariable{variable: Variable::new(3), value: Operand::new_var(1)},

            // detect snd position
            ZOP::ReadChar{local_var_id: 2},
            // if enter, then we are finished
            ZOP::JE{operand1: Operand::new_var(2), operand2: Operand::new_const(13), jump_to_label: "system_check_links_load_link_address".to_string()},
            ZOP::Sub{operand1: Operand::new_var(2), operand2: Operand::new_const(48), save_variable: Variable::new(2)},
            ZOP::PrintNumVar{variable: Variable::new(2)},
            // check if the the detected key is > 9
            ZOP::JG{operand1: Operand::new_var(2), operand2: Operand::new_const(9), jump_to_label: "system_check_links_error".to_string()},
            // check if key < 0
            ZOP::JL{operand1: Operand::new_var(2), operand2: Operand::new_const(0), jump_to_label: "system_check_links_error".to_string()},
            // calculates the the number of the frst position*10 + number of the
            // snd position
            // first position, so multiply with 10
            ZOP::Mul{operand1: Operand::new_var(1), operand2: Operand::new_const(10), save_variable: Variable::new(3)},
            ZOP::Add{operand1: Operand::new_var(3), operand2: Operand::new_var(2), save_variable: Variable::new(3)},
            // check if the the calculated number > number of link
            ZOP::JG{operand1: Operand::new_var(3), operand2: Operand::new_var(16), jump_to_label: "system_check_links_error".to_string()},

            ZOP::Jump{jump_to_label: "system_check_links_load_link_address".to_string()},
            // error
            ZOP::Label{name: "system_check_links_error".to_string()},
            ZOP::Newline,
            ZOP::Print{text: "Not a valid link, try again: ".to_string()},
            ZOP::Jump{jump_to_label: "system_check_links_more_than_9".to_string()},

            // loads the address of the link from the array
            ZOP::Label{name: "system_check_links_load_link_address".to_string()},
            ZOP::SetTextStyle{bold: false, reverse: false, monospace: false, italic: false},
            // decrement 0x03 becouse the array starts at 0 and not at 1
            ZOP::Dec{variable: 3},
            ZOP::LoadW{array_address: Operand::new_large_const(save_at_addr as i16), index: Variable::new(3), variable: Variable::new(2)},

            // no more links exist
            ZOP::StoreVariable{variable: Variable::new(16), value: Operand::new_const(0)},
            ZOP::Newline,

            // clears window bevor jumping
            ZOP::EraseWindow{value: -1},

            // jump to the new passage
            ZOP::Call1NVar{variable: 0x02},
            ZOP::Label{name: "system_check_links_end_ret".to_string()},
            ZOP::Ret{value: Operand::new_const(0)},

            ZOP::Label{name: "system_check_links_end_quit".to_string()},
            ZOP::Quit
        ]);
    }

    /// Easter-egg, with konami-code to start.
    pub fn routine_check_more(&mut self) {
        if self.easter_egg {
            self.emit(vec![
                ZOP::Routine{name: "system_check_more".to_string(), count_variables: 1},
                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(129), jump_to_label: "system_check_more_ko_1".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_1".to_string()},

                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(130), jump_to_label: "system_check_more_ko_2".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_2".to_string()},

                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(130), jump_to_label: "system_check_more_ko_3".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_3".to_string()},

                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(131), jump_to_label: "system_check_more_ko_4".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_4".to_string()},

                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(132), jump_to_label: "system_check_more_ko_5".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_5".to_string()},

                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(131), jump_to_label: "system_check_more_ko_6".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_6".to_string()},

                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(132), jump_to_label: "system_check_more_ko_7".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_7".to_string()},

                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(98), jump_to_label: "system_check_more_ko_8".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_8".to_string()},

                ZOP::ReadChar{local_var_id: 0x01},
                ZOP::JE{operand1: Operand::new_var(0x01), operand2: Operand::new_const(97), jump_to_label: "system_check_more_ko_9".to_string()},
                ZOP::Ret{value: Operand::new_const(0)},
                ZOP::Label{name: "system_check_more_ko_9".to_string()},
                ZOP::Call1N{jump_to_label: "easter_egg_start".to_string()},
                ZOP::Ret{value: Operand::new_const(0)}
            ]);
            routine_easteregg(self);
        } else {
            self.emit(vec![
                ZOP::Routine{name: "system_check_more".to_string(), count_variables: 1},
                ZOP::Ret{value: Operand::new_const(0)}
            ]);
        }
    }

    /// Print UTF-16 string at addr.
    ///
    /// Expects an address as argument where the first u16 stored is the length of the string as the
    /// number of u16 chars, followed by the string to print.
    /// This only works if the string is within the address space up to 0xffff.
    pub fn routine_print_unicode(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "print_unicode".to_string(), count_variables: 4},
            // DEBUG    ZOP::Print{text: "pos:".to_string()}, ZOP::PrintNumVar{variable: 0x01},
            // addr as arg1 in 0x01, copy length to 0x02
            ZOP::LoadW{array_address: Operand::new_var(1), index: Variable::new(4), variable: Variable::new(2)},  // index at var:4 is 0
            ZOP::JE{operand1: Operand::new_var(2), operand2: Operand::new_large_const(0), jump_to_label: "inter_char_end".to_string()},
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
            ZOP::Label{name: "inter_char_end".to_string()},
            ZOP::Ret{value: Operand::new_const(0)}
        ]);
    }

    /// Update the cursor position in the global cursor_pos variable.
    pub fn update_cursor_pos(&mut self) {
        let cursor_pos = self.cursor_pos;
        self.emit(vec![ZOP::GetCursor{store_addr: Operand::new_large_const(cursor_pos as i16)}]);
    }

    /// Needed to simulate a javascript browser input dialog, receives a prompt message and a
    /// default value as string arguments.
    pub fn routine_prompt(&mut self) {
        let msg = Variable::new(1); // arg1  displayed message
        let msg_op = Operand::new_var(msg.id);
        let val = Variable::new(2); // arg2  current input value
        let val_op = Operand::new_var(val.id);
        let c = Variable::new(3);  // read character
        let c_op = Operand::new_var(c.id);
        let t = Variable::new(4);  // tmp
        let t_op = Operand::new_var(t.id);
        let z = Variable::new(5);  // tmp
        let z_op = Operand::new_var(z.id);
        let a = Variable::new(6);  // tmp
        let a_op = Operand::new_var(a.id);
        // let cursor_pos = self.cursor_pos;  see TODO at end of function
        self.emit(vec![
            ZOP::Routine{name: "rt_prompt".to_string(), count_variables: 6},
            // read length of default value to a and copy the default value so that we only work on the copy
            ZOP::LoadW{array_address: val_op.clone(), index: a.clone(), variable: a.clone()},
            ZOP::StoreVariable{variable: t.clone(), value: val_op.clone()},
            ZOP::Inc{variable: a.id},
            ZOP::Call2S{jump_to_label: "malloc".to_string(), arg: a_op.clone(), result: val.clone()},
            ZOP::Dec{variable: a.id},
            ZOP::StoreW{array_address: val_op.clone(), index: z.clone(), variable: a.clone()},
            ZOP::StoreVariable{variable: z.clone(), value: val_op.clone()},
            ZOP::Inc{variable: z.id},
            ZOP::Inc{variable: z.id},
            ZOP::CallVNA2{jump_to_label: "strcpy".to_string(), arg1: t_op.clone(), arg2: z_op.clone()},
            ZOP::PrintUnicodeStr{address: msg_op.clone()},
            ZOP::Newline,
            ZOP::Print{text: "> ".to_string()},
            ZOP::PrintUnicodeStr{address: val_op.clone()},
            ZOP::Label{name: "rt_prompt_loop".to_string()},
            ZOP::ReadChar{local_var_id: c.id},
            // on backspace
            ZOP::JE{operand1: c_op.clone(), operand2: Operand::new_const(8), jump_to_label: "rt_prompt_del".to_string()},
            // on enter:
            ZOP::JE{operand1: c_op.clone(), operand2: Operand::new_const(13), jump_to_label: "rt_prompt_return".to_string()},
            ZOP::PrintUnicodeVar{var: c.clone()},
            // add strings:
            // make string of length 1 for c
            ZOP::Call2S{jump_to_label: "malloc".to_string(), arg: Operand::new_const(2), result: t.clone()},
            ZOP::StoreVariable{variable: z.clone(), value: Operand::new_large_const(1)},
            ZOP::StoreVariable{variable: a.clone(), value: Operand::new_large_const(0)},
            ZOP::StoreW{array_address: t_op.clone(), index: a.clone(), variable: z.clone()},
            ZOP::StoreW{array_address: t_op.clone(), index: z.clone(), variable: c.clone()},
            ZOP::StoreVariable{variable: z.clone(), value: val_op.clone()},
            // make new string and remeber strings to delete in z and t
            ZOP::CallVSA2{jump_to_label: "strcat".to_string(), arg1: val_op.clone(), arg2: t_op.clone(), result: val.clone()},
            // free them manually as we can't wait for the garbage collector
            ZOP::Call2NWithArg{jump_to_label: "manual_free".to_string(), arg: t_op.clone()},
            ZOP::Call2NWithArg{jump_to_label: "manual_free".to_string(), arg: z_op.clone()},
            ZOP::Jump{jump_to_label: "rt_prompt_loop".to_string()},
            ZOP::Label{name: "rt_prompt_del".to_string()},
            ZOP::StoreVariable{variable: a.clone(), value: Operand::new_large_const(0)},
            ZOP::LoadW{array_address: val_op.clone(), index: a.clone(), variable: a.clone()},
            // jump back if length is 0
            ZOP::JE{operand1: a_op.clone(), operand2: Operand::new_const(0), jump_to_label: "rt_prompt_loop".to_string()},
            // otherwise set last u16 to -1 in order to free it
            ZOP::StoreVariable{variable: t.clone(), value: Operand::new_large_const(-1i16)},
            ZOP::StoreW{array_address: val_op.clone(), index: a.clone(), variable: t.clone()},
            ZOP::Dec{variable: a.id},
            // reduce length of string by 1
            ZOP::StoreVariable{variable: t.clone(), value: Operand::new_large_const(0)},
            ZOP::StoreW{array_address: val_op.clone(), index: t.clone(), variable: a.clone()},
            // @TODO: these two commands should go to the beginning of the line and erase it before we print again,
            // but rightnow it does not work and behaves strange. this is why we have a Newline here instead
            // ZOP::UpdateCursorPos,
            // read current row
            // ZOP::LoadW{array_address: Operand::new_large_const(cursor_pos as i16), index: t.clone(), variable: a.clone()},
            // ZOP::SetCursorOperand{row: a_op.clone(), col: Operand::new_const(1)},
            // ZOP::EraseLine,
            ZOP::EraseWindow{value: -1},
            ZOP::PrintUnicodeStr{address: msg_op.clone()},
            ZOP::Newline,
            ZOP::Print{text: "> ".to_string()},
            ZOP::PrintUnicodeStr{address: val_op.clone()},
            ZOP::Jump{jump_to_label: "rt_prompt_loop".to_string()},
            ZOP::Label{name: "rt_prompt_return".to_string()},
            ZOP::Newline,
            ZOP::Ret{value: val_op},
        ]);
    }

    /// malloc Z-Routine: Allocate a specified number of words of dynamic memory.
    ///
    /// `argument`: amount of u16 to allocate
    ///
    /// After receiving the address you are requested to write down the
    /// number of u16 you are actually using in the first u16 and then
    /// if you ever want to decrease this, you have to write -1i16 at
    /// the 'freed' u16s at the end. increasing it is not allowed.
    /// memory will be freed after each passage if there is no global
    /// variable pointing to it.
    pub fn routine_malloc(&mut self) {
        let heap_start = self.heap_start;
        let static_addr = self.static_addr - 2; // we'll write u16 before static_addr where we
                                                // store the maximum of upper bounds of allocations
                                                // so that the garbage collector does not need to clean
                                                // if the memory was untouched
        self.emit(vec![
            ZOP::Routine{name: "malloc".to_string(), count_variables: 15},
            // var1 is the allocation size given in needed amount of u16
            // var4 is the possible memory address
            // var2 contains entry at index var3 of var4
            // var3 is index on array at var4
            // var5 has the upper boundary for var4 which is at static_addr-length*2
            // var6 contains the need_to_clean_up_to entry
            // var7 is used for temporary calculation of the pointer within the possible alloc block
            // init var4 with heap_start
            ZOP::StoreVariable{variable: Variable::new(4), value: Operand::new_large_const(heap_start as i16)},
            // calc var5
            ZOP::StoreVariable{variable: Variable::new(5), value: Operand::new_large_const(static_addr as i16)},
            ZOP::Sub{operand1: Operand::new_var(5), operand2: Operand::new_var(1), save_variable: Variable::new(5)},
            ZOP::Sub{operand1: Operand::new_var(5), operand2: Operand::new_var(1), save_variable: Variable::new(5)},
            // load need_to_clean_up_to
            ZOP::LoadW{array_address: Operand::new_large_const(static_addr as i16), index: Variable::new(6), variable: Variable::new(6)},
            ZOP::Label{name: "malloc_loop".to_string()},
            // check if we have to give up and quit
            ZOP::JE{operand1: Operand::new_var(4), operand2: Operand::new_var(5), jump_to_label: "malloc_fail".to_string()},
            // check if we are behind highest allocated block and do not need to check if it was freed
            ZOP::JE{operand1: Operand::new_var(4), operand2: Operand::new_var(6), jump_to_label: "malloc_return".to_string()},
            // set var3 index to 0
            ZOP::StoreVariable{variable: Variable::new(3), value: Operand::new_large_const(0)},
            // read the entry of var4 at pos var3 to var2
            ZOP::LoadW{array_address: Operand::new_var(4), index: Variable::new(3), variable: Variable::new(2)},
            // jump to malloc_is_free if entry is free
            ZOP::JL{operand1: Operand::new_var(2), operand2: Operand::new_large_const(0), jump_to_label: "malloc_is_free".to_string()},
            // length of entry is >= 0 so now we skip length*2 (content) and go to the next entry after it by adding 2 to skip one u16
            ZOP::Add{operand1: Operand::new_var(4), operand2: Operand::new_large_const(2), save_variable: Variable::new(4)},
            ZOP::Add{operand1: Operand::new_var(4), operand2: Operand::new_var(2), save_variable: Variable::new(4)},
            ZOP::Add{operand1: Operand::new_var(4), operand2: Operand::new_var(2), save_variable: Variable::new(4)},
            ZOP::Jump{jump_to_label: "malloc_loop".to_string()},
            ZOP::Label{name: "malloc_is_free".to_string()},
            // if var3 is equal the allocation size, we have found enough space at var4 and can return it
            ZOP::JE{operand1: Operand::new_var(3), operand2: Operand::new_var(1), jump_to_label: "malloc_return".to_string()},
            // or if we reached last upper alloc bound
            ZOP::JE{operand1: Operand::new_var(4), operand2: Operand::new_var(6), jump_to_label: "malloc_return".to_string()},
            ZOP::Inc{variable: 3},  // increase index
            // check if we have to give up and quit
            ZOP::JE{operand1: Operand::new_var(4), operand2: Operand::new_var(5), jump_to_label: "malloc_fail".to_string()},
            // load entry of var4 at pos var3 to var2
            ZOP::LoadW{array_address: Operand::new_var(4), index: Variable::new(3), variable: Variable::new(2)},
            // check if we reached last upper alloc bound by calculation var7 as the current position in possible alloc block
            ZOP::Add{operand1: Operand::new_var(4), operand2: Operand::new_var(3), save_variable: Variable::new(7)},
            ZOP::Add{operand1: Operand::new_var(7), operand2: Operand::new_var(3), save_variable: Variable::new(7)},
            ZOP::JE{operand1: Operand::new_var(7), operand2: Operand::new_var(6), jump_to_label: "malloc_return".to_string()},
            // continue testing for free memory if this one was free
            ZOP::JL{operand1: Operand::new_var(2), operand2: Operand::new_large_const(0), jump_to_label: "malloc_is_free".to_string()},
            // otherwise set var4 to the actual position (var4+2*var3) and start from beginning because we have to jump over this entry
            ZOP::Add{operand1: Operand::new_var(4), operand2: Operand::new_var(3), save_variable: Variable::new(4)},
            ZOP::Add{operand1: Operand::new_var(4), operand2: Operand::new_var(3), save_variable: Variable::new(4)},
            ZOP::Jump{jump_to_label: "malloc_loop".to_string()},
            ZOP::Label{name: "malloc_return".to_string()},
            // save upper bound to the last u16 before (real) static_addr
            // add up allocation address and allocation length*2 (as it is amount of u16)
            ZOP::Add{operand1: Operand::new_var(4), operand2: Operand::new_var(1), save_variable: Variable::new(2)},
            ZOP::Add{operand1: Operand::new_var(2), operand2: Operand::new_var(1), save_variable: Variable::new(2)},
            // only set need_to_clean_up_to entry if we allocated behind it
            ZOP::JL{operand1: Operand::new_var(2), operand2: Operand::new_var(6), jump_to_label: "malloc_return_not_set_need_to_clean_up".to_string()},
            ZOP::StoreVariable{variable: Variable::new(3), value: Operand::new_const(0)},
            ZOP::StoreW{array_address: Operand::new_large_const(static_addr as i16), index: Variable::new(3), variable: Variable::new(2)},
            ZOP::Label{name: "malloc_return_not_set_need_to_clean_up".to_string()},
            // return allocation addr
            ZOP::Ret{value: Operand::new_var(4)},
            ZOP::Label{name: "malloc_fail".to_string()},
            ZOP::Print{text: "MALLOC-FAIL".to_string()},
            ZOP::Quit,
        ]);
    }

    /// strcpy Z-Routine: Copy a string.
    ///
    /// first argument is pointer to utf16 string containing length at first u16
    /// second the the destination address in memory where the string is copied to,
    /// while the first length u16 is not copied.
    pub fn routine_strcpy(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "strcpy".to_string(), count_variables: 15},
            // var1 has the from_addr where first u16 is the length
            // var2 has the to_addr where we do *not* write the length in the first u16
            // var4 is the index and equals to number of u16 written
            // var5 has the character to copy
            // load length to var3
            ZOP::LoadW{array_address: Operand::new_var(1), index: Variable::new(4), variable: Variable::new(3)},
            ZOP::Inc{variable: 1}, ZOP::Inc{variable: 1},  // point to first source byte
            ZOP::Label{name: "strcpy_loop".to_string()},
            ZOP::JE{operand1: Operand::new_var(4), operand2: Operand::new_var(3), jump_to_label: "strcpy_return".to_string()},
            ZOP::LoadW{array_address: Operand::new_var(1), index: Variable::new(4), variable: Variable::new(5)},
            ZOP::StoreW{array_address: Operand::new_var(2), index: Variable::new(4), variable: Variable::new(5)},
            ZOP::Inc{variable: 4},  // point to next byte at dest and source
            ZOP::Jump{jump_to_label: "strcpy_loop".to_string()},
            ZOP::Label{name: "strcpy_return".to_string()},
            ZOP::Ret{value: Operand::new_const(0)}
        ]);
    }

    /// strcat Z-Routine: Concatenate two strings.
    ///
    /// returns a reference to a string concatenation of the first and second string parameters.
    pub fn routine_strcat(&mut self) {
        let addr1 = Variable::new(1);
        let addr2 = Variable::new(2);
        let len1 = Variable::new(3);
        let len2 = Variable::new(4);
        let tmp = Variable::new(5);
        let save_var = Variable::new(6);
        self.emit(vec![
            ZOP::Routine{name: "strcat".to_string(), count_variables: 15},
            // var1 has the first str-addr, var2 the second str-addr
            // set to 0 for index access
            ZOP::StoreVariable{variable: len1.clone(), value: Operand::new_large_const(0)},
            // read length of string1 which is stored at index 0
            ZOP::LoadW{array_address: Operand::new_var(addr1.id), index: len1.clone(), variable: len1.clone()},
            // set to 0 for index access
            ZOP::StoreVariable{variable: len2.clone(), value: Operand::new_large_const(0)},
            // read length of string2 which is stored at index 0
            ZOP::LoadW{array_address: Operand::new_var(addr2.id), index: len2.clone(), variable: len2.clone()},
            // store new length = len1+len2 in save_var
            ZOP::StoreVariable{variable: save_var.clone(), value: Operand::new_var(len1.id)},
            ZOP::Add{operand1: Operand::new_var(len2.id), operand2: Operand::new_var(save_var.id), save_variable: save_var.clone()},
            ZOP::Inc{variable: save_var.id},  // increase as we will also save the length at first u16
            ZOP::Call2S{jump_to_label: "malloc".to_string(), arg: Operand::new_var(save_var.id), result: save_var.clone()},
            // write len1+len2 to len2
            ZOP::Add{operand1: Operand::new_var(len1.id), operand2: Operand::new_var(len2.id), save_variable: len2.clone()},
            // set tmp to 0 for array index 0
            ZOP::StoreVariable{variable: tmp.clone(), value: Operand::new_large_const(0)},
            // and store len1+len2 in first u16
            ZOP::StoreW{array_address: Operand::new_var(save_var.id), index: tmp.clone(), variable: len2.clone()},
            // set tmp to save_var_addr+2
            ZOP::StoreVariable{variable: tmp.clone(), value: Operand::new_large_const(2)},
            ZOP::Add{operand1: Operand::new_var(tmp.id), operand2: Operand::new_var(save_var.id), save_variable: tmp.clone()},
            // strcopy (addr1 to save_var_addr+2)
            ZOP::CallVNA2{jump_to_label: "strcpy".to_string(), arg1: Operand::new_var(addr1.id), arg2: Operand::new_var(tmp.id)},
            // set tmp to save_var_addr+2+len1*2
            ZOP::Add{operand1: Operand::new_var(tmp.id), operand2: Operand::new_var(len1.id), save_variable: tmp.clone()},
            ZOP::Add{operand1: Operand::new_var(tmp.id), operand2: Operand::new_var(len1.id), save_variable: tmp.clone()},
            // strcopy (addr2 to save_var_addr+2+len1*2)
            ZOP::CallVNA2{jump_to_label: "strcpy".to_string(), arg1: Operand::new_var(addr2.id), arg2: Operand::new_var(tmp.id)},
            ZOP::Ret{value: Operand::new_var(save_var.id)}
        ]);
    }

    /// strcmp Z-Routine: Compare two strings.
    ///
    /// returns 0 if both given strings are equal and -1 if the first is
    /// alphabetically smaller than the second and +1 vice versa.
    pub fn routine_strcmp(&mut self) {
        let addr1 = Variable::new(1);
        let addr2 = Variable::new(2);
        let len1 = Variable::new(3);
        let len2 = Variable::new(4);
        let count = Variable::new(5);
        let c1 = Variable::new(6);
        let c2 = Variable::new(7);
        self.emit(vec![
            ZOP::Routine{name: "strcmp".to_string(), count_variables: 15},
            // var1 has the first str-addr, var2 the second str-addr
            // set to 0 for index access
            ZOP::StoreVariable{variable: count.clone(), value: Operand::new_large_const(0)},
            // read length of string1 which is stored at index 0
            ZOP::LoadW{array_address: Operand::new_var(addr1.id), index: count.clone(), variable: len1.clone()},
            // read length of string2 which is stored at index 0
            ZOP::LoadW{array_address: Operand::new_var(addr2.id), index: count.clone(), variable: len2.clone()},
            // handle case that one has length 0 so that we do not enter the loop
            ZOP::JE{operand1: Operand::new_var(len1.id), operand2: Operand::new_large_const(0), jump_to_label: "strcmp_firstzero".to_string()},
            ZOP::JE{operand1: Operand::new_var(len2.id), operand2: Operand::new_large_const(0), jump_to_label: "strcmp_secondzero".to_string()},
            ZOP::Label{name: "strcmp_loop".to_string()},
            ZOP::Inc{variable: count.id},
            // check if one of the strings ended, then see whether one is longer in _fristzero/_secondzero
            ZOP::JG{operand1: Operand::new_var(count.id), operand2: Operand::new_var(len1.id), jump_to_label: "strcmp_firstzero".to_string()},
            ZOP::JG{operand1: Operand::new_var(count.id), operand2: Operand::new_var(len2.id), jump_to_label: "strcmp_secondzero".to_string()},
            // read the two characters
            ZOP::LoadW{array_address: Operand::new_var(addr1.id), index: count.clone(), variable: c1.clone()},
            ZOP::LoadW{array_address: Operand::new_var(addr2.id), index: count.clone(), variable: c2.clone()},
            // compare them
            ZOP::JG{operand1: Operand::new_var(c1.id), operand2: Operand::new_var(c2.id), jump_to_label: "strcmp_greater".to_string()},
            ZOP::JL{operand1: Operand::new_var(c1.id), operand2: Operand::new_var(c2.id), jump_to_label: "strcmp_lesser".to_string()},
            ZOP::Jump{jump_to_label: "strcmp_loop".to_string()},
            ZOP::Label{name: "strcmp_firstzero".to_string()},
            ZOP::JE{operand1: Operand::new_var(len1.id), operand2: Operand::new_var(len2.id), jump_to_label: "strcmp_equal".to_string()},
            ZOP::Ret{value: Operand::new_large_const(-1)},
            ZOP::Label{name: "strcmp_secondzero".to_string()},
            ZOP::JE{operand1: Operand::new_var(len1.id), operand2: Operand::new_var(len2.id), jump_to_label: "strcmp_equal".to_string()},
            ZOP::Ret{value: Operand::new_large_const(1)},
            ZOP::Label{name: "strcmp_equal".to_string()},
            ZOP::Ret{value: Operand::new_large_const(0)},
            ZOP::Label{name: "strcmp_lesser".to_string()},
            ZOP::Ret{value: Operand::new_large_const(-1)},
            ZOP::Label{name: "strcmp_greater".to_string()},
            ZOP::Ret{value: Operand::new_large_const(1)},
        ]);
    }

    /// malloc_init Z-Routine: Initialize the dynamic memory.
    pub fn routine_malloc_init(&mut self) {
        let heap_start = self.heap_start;
        let static_addr = self.static_addr - 2;  // store last alloc upper bound as u16 before static_addr
        self.emit(vec![
            ZOP::Routine{name: "malloc_init".to_string(), count_variables: 4},
            // var3 stays 0
            // heap_start is in var1 and will be increased during loop
            // var2 stays -1
            ZOP::StoreVariable{variable: Variable::new(1), value: Operand::new_large_const(heap_start as i16)},
            // write heap start as last used addr
            ZOP::StoreW{array_address: Operand::new_large_const(static_addr as i16), index: Variable::new(3), variable: Variable::new(1)},
            // init with -1 not needed as we use need_to_clean_up_to entry
            //ZOP::StoreVariable{variable: Variable::new(2), value: Operand::new_large_const(-1i16)},
            //ZOP::Label{name: "malloc_init_loop".to_string()},
            //ZOP::StoreW{array_address: Operand::new_var(1), index: Variable::new(3), variable: Variable::new(2)},
            //ZOP::Inc{variable: 1}, ZOP::Inc{variable: 1},
            //ZOP::JNE{operand1: Operand::new_var(1), operand2: Operand::new_large_const(static_addr as i16), jump_to_label: "malloc_init_loop".to_string()},
            ZOP::Ret{value: Operand::new_const(0)}
        ]);
    }

    /// mem_free Z-Routine: Free unused dynamic memory.
    ///
    /// This is implemented as a simple tracing garbage collector.
    pub fn routine_mem_free(&mut self) {
        let heap_start = self.heap_start;
        let static_addr = self.static_addr - 2;  // the last u16 contains the highest addr of allocated space
        let global_addr = self.global_addr;
        let type_store = self.type_store;
        let pos = Variable::new(1);
        let zero = Variable::new(2);
        let c = Variable::new(3);
        let m = Variable::new(4);
        let t = Variable::new(5);
        let varid = Variable::new(6);
        let varcontent = Variable::new(7);
        let need_to_clean_up_to = Variable::new(8);  // @IMPROVEMENT: consider reducing it again if last element was freed
        self.emit(vec![
            ZOP::Routine{name: "mem_free".to_string(), count_variables: 15},
            ZOP::LoadW{array_address: Operand::new_large_const(static_addr as i16), index: zero.clone(), variable: need_to_clean_up_to.clone()},
            // set m to -1
            ZOP::StoreVariable{variable: m.clone(), value: Operand::new_large_const(-1i16)},
            // set pos to current position
            ZOP::StoreVariable{variable: pos.clone(), value: Operand::new_large_const(heap_start as i16)},
            ZOP::Dec{variable: pos.id},
            ZOP::Dec{variable: pos.id},
            ZOP::Label{name: "mem_free_loop".to_string()},
            ZOP::Inc{variable: pos.id},
            ZOP::Inc{variable: pos.id},
            // exit at end of mem
            ZOP::JE{operand1: Operand::new_var(pos.id), operand2: Operand::new_large_const(static_addr as i16), jump_to_label: "mem_free_exit".to_string()},
            // or also exit at end of up-to-now allocated memory
            ZOP::JE{operand1: Operand::new_var(pos.id), operand2: Operand::new_var(need_to_clean_up_to.id), jump_to_label: "mem_free_exit".to_string()},
            // read entry to c
            ZOP::LoadW{array_address: Operand::new_var(pos.id), index: zero.clone(), variable: c.clone()},
            // continue search if entry is free
            ZOP::JL{operand1: Operand::new_var(c.id), operand2: Operand::new_large_const(0), jump_to_label: "mem_free_loop".to_string()},
            // ZOP::PrintNumVar{variable: pos.clone()},
            // ZOP::Print{text: "CHECK".to_string()},
            // ZOP::PrintNumVar{variable: c.clone()},
            // start loop for checking and init varid to iterate on
            ZOP::StoreVariable{variable: varid.clone(), value: Operand::new_large_const(15i16)},
            ZOP::Label{name: "mem_free_check".to_string()},
            ZOP::Inc{variable: varid.id},
            ZOP::LoadW{array_address: Operand::new_large_const(global_addr as i16 - 32i16), index: varid.clone(), variable: varcontent.clone()},
            // ZOP::PrintNumVar{variable: varid.clone()}, ZOP::Print{text: ":".to_string()},
            // ZOP::PrintNumVar{variable: varcontent.clone()},
            // ZOP::Print{text: " ".to_string()},
            // check if entry at pos is not referenced by a global variable, then we free it, otherwise jump down
            ZOP::JE{operand1: Operand::new_var(pos.id), operand2: Operand::new_var(varcontent.id), jump_to_label: "mem_free_continue".to_string()},
            ZOP::JL{operand1: Operand::new_var(varid.id), operand2: Operand::new_large_const(255i16), jump_to_label: "mem_free_check".to_string()},
            // finished loop for checking
            // set t to position after the whole entry so now we skip length*2 (content)
            ZOP::Add{operand1: Operand::new_var(pos.id), operand2: Operand::new_var(c.id), save_variable: t.clone()},
            ZOP::Add{operand1: Operand::new_var(t.id), operand2: Operand::new_var(c.id), save_variable: t.clone()},
            ZOP::Dec{variable: pos.id},
            ZOP::Dec{variable: pos.id},
            // ZOP::Print{text: "DELETE".to_string()},
            ZOP::Label{name: "mem_free_delete".to_string()},
            // continue until pos is at position t
            ZOP::JE{operand1: Operand::new_var(pos.id), operand2: Operand::new_var(t.id), jump_to_label: "mem_free_loop".to_string()},
            ZOP::Inc{variable: pos.id},
            ZOP::Inc{variable: pos.id},
            // exit at end of mem
            ZOP::JE{operand1: Operand::new_var(pos.id), operand2: Operand::new_large_const(static_addr as i16), jump_to_label: "mem_free_exit".to_string()},
            // write -1 to pos
            ZOP::StoreW{array_address: Operand::new_var(pos.id), index: zero.clone(), variable: m.clone()},
            ZOP::Jump{jump_to_label: "mem_free_delete".to_string()},
            ZOP::Label{name: "mem_free_continue".to_string()},
            // ZOP::Print{text: "IS-USED".to_string()},
            // mem is not free but tells us the length of the entry
            // length of entry is >= 0 so now we skip length*2 (content)
            ZOP::Add{operand1: Operand::new_var(pos.id), operand2: Operand::new_var(c.id), save_variable: pos.clone()},
            ZOP::Add{operand1: Operand::new_var(pos.id), operand2: Operand::new_var(c.id), save_variable: pos.clone()},
            ZOP::Jump{jump_to_label: "mem_free_loop".to_string()},
            ZOP::Label{name: "mem_free_exit".to_string()},
            // set type entries variables 0-15 of type_store to 0 for no type
            ZOP::StoreVariable{variable: pos.clone(), value: Operand::new_large_const(0)},
            ZOP::Label{name: "mem_free_uninit_local_var_types".to_string()},
            ZOP::StoreB{array_address: Operand::new_large_const(type_store as i16), index: pos.clone(), variable: zero.clone()},
            ZOP::Inc{variable: pos.id},
            ZOP::JL{operand1: Operand::new_var(pos.id), operand2: Operand::new_large_const(16i16), jump_to_label: "mem_free_uninit_local_var_types".to_string()},
            ZOP::Ret{value: Operand::new_const(0)}
        ]);
    }

    /// manual_free Z-Routine: manual free call to erase used heap memory if you can not wait for
    /// the GC.
    pub fn routine_manual_free(&mut self) {
        let addr_op = Operand::new_var(1);
        let index = Variable::new(2);
        let index_op = Operand::new_var(index.id);
        let length = Variable::new(3);
        let length_op = Operand::new_var(length.id);
        let del = Variable::new(4);
        self.emit(vec![
            ZOP::Routine{name: "manual_free".to_string(), count_variables: 4},
            ZOP::StoreVariable{variable: del.clone(), value: Operand::new_large_const(-1i16)},
            // load length
            ZOP::LoadW{array_address: addr_op.clone(), index: index.clone(), variable: length.clone()},
            ZOP::Label{name: "manual_free_loop".to_string()},
            ZOP::StoreW{array_address: addr_op.clone(), index: index.clone(), variable: del.clone()},
            ZOP::Inc{variable: index.id},
            ZOP::JLE{operand1: index_op.clone(), operand2: length_op.clone(), jump_to_label: "manual_free_loop".to_string()},
            ZOP::Ret{value: Operand::new_const(0)},
        ]);
    }

    /// itoa Z-Routine: Convert an int to a string.
    ///
    /// convert from number at arg1 to string at base of 10, returns the str addr.
    pub fn routine_itoa(&mut self) {
        let number = Variable::new(1);  // var1 is the given number
        let stra = Variable::new(2);  // the result string
        let i = Variable::new(3);  // the current index
        let zero = Variable::new(6);  // var14 stays 0
        let tmp = Variable::new(4);
        let z = Variable::new(5);
        self.emit(vec![
            ZOP::Routine{name: "itoa".to_string(), count_variables: 7},
            // set first digit we handle to 10000
            ZOP::StoreVariable{variable: z.clone(), value: Operand::new_large_const(10000i16)},
            // maximum length is 7 characters like -12345 and length=6 at first u16
            ZOP::Call2S{jump_to_label: "malloc".to_string(), arg: Operand::new_large_const(7), result: stra.clone()},
            ZOP::Inc{variable: i.id},  // point at first character to be written
            // write '-' if < 0
            ZOP::JGE{operand1: Operand::new_var(number.id), operand2: Operand::new_large_const(0), jump_to_label: "itoa_write".to_string()},
            ZOP::StoreVariable{variable: tmp.clone(), value: Operand::new_large_const('-' as i16)},
            ZOP::StoreW{array_address: Operand::new_var(stra.id), index: i.clone(), variable: tmp.clone()},
            ZOP::Inc{variable: i.id}, // go to next position
            // and make number positive from now on (max 32767)
            ZOP::Mul{operand1: Operand::new_large_const(-1i16), operand2: Operand::new_var(number.id), save_variable: number.clone()},
            ZOP::Label{name: "itoa_write".to_string()},
            // tmp=number/z
            ZOP::Div{operand1: Operand::new_var(number.id), operand2: Operand::new_var(z.id), save_variable: tmp.clone()},
            // do not write if digit is 0
            ZOP::JE{operand1: Operand::new_var(tmp.id), operand2: Operand::new_large_const(0i16), jump_to_label: "itoa_continue".to_string()},
            // write digit tmp as utf16
            ZOP::Add{operand1: Operand::new_large_const('0' as i16), operand2: Operand::new_var(tmp.id), save_variable: tmp.clone()},
            ZOP::StoreW{array_address: Operand::new_var(stra.id), index: i.clone(), variable: tmp.clone()},
            ZOP::Inc{variable: i.id}, // go to next position
            ZOP::Label{name: "itoa_continue".to_string()},
            // number=number % z
            ZOP::Mod{operand1: Operand::new_var(number.id), operand2: Operand::new_var(z.id), save_variable: number.clone()},
            // continue with z/10
            ZOP::Div{operand1: Operand::new_var(z.id), operand2: Operand::new_large_const(10i16), save_variable: z.clone()},
            ZOP::JG{operand1: Operand::new_var(z.id), operand2: Operand::new_large_const(1i16), jump_to_label: "itoa_write".to_string()},
            // write number as utf16 as it is in range 0-9
            ZOP::Add{operand1: Operand::new_large_const('0' as i16), operand2: Operand::new_var(number.id), save_variable: tmp.clone()},
            ZOP::StoreW{array_address: Operand::new_var(stra.id), index: i.clone(), variable: tmp.clone()},
            // write length i at first position
            ZOP::StoreW{array_address: Operand::new_var(stra.id), index: zero.clone(), variable: i.clone()},
            ZOP::Ret{value: Operand::new_var(stra.id)}
        ]);
    }

    /// helper function to print out the content of a variable according to its type.
    pub fn routine_print_var(&mut self) {
        let varid = Variable::new(1);  // first argument
        let varcontent = Variable::new(2);  // second argument
        let vartype = Variable::new(3);
        let type_store = self.type_store;
        self.emit(vec![
            ZOP::Routine{name: "print_var".to_string(), count_variables: 4},
            // get vartype
            ZOP::LoadBOperand{array_address: Operand::new_large_const(type_store as i16), index: Operand::new_var(varid.id), variable: vartype.clone()},
            ZOP::JE{operand1: Operand::new_var(vartype.id), operand2: Operand::new_const(Type::String as u8), jump_to_label: "print_var_string".to_string()},
            ZOP::JE{operand1: Operand::new_var(vartype.id), operand2: Operand::new_const(Type::Bool as u8), jump_to_label: "print_var_bool".to_string()},
            // print number
            ZOP::PrintNumVar{variable: varcontent.clone()},
            ZOP::Ret{value: Operand::new_const(0)},
            ZOP::Label{name: "print_var_bool".to_string()},
            ZOP::JE{operand1: Operand::new_var(varcontent.id), operand2: Operand::new_const(0), jump_to_label: "print_var_boolfalse".to_string()},
            ZOP::Print{text: "true".to_string()},
            ZOP::Ret{value: Operand::new_const(0)},
            ZOP::Label{name: "print_var_boolfalse".to_string()},
            ZOP::Print{text: "false".to_string()},
            ZOP::Ret{value: Operand::new_const(0)},
            ZOP::Label{name: "print_var_string".to_string()},
            // print var string
            ZOP::PrintUnicodeStr{address: Operand::new_var(varcontent.id)},
            ZOP::Ret{value: Operand::new_const(0)},
        ]);
    }

    /// Print a variable.
    fn print_var(&mut self, variable: &Variable) {
        self.emit(vec![
            ZOP::CallVNA2{jump_to_label: "print_var".to_string(), arg1: Operand::new_const(variable.id), arg2: Operand::new_var(variable.id)},
        ]);
    }

    /// Changes the variable type.
    fn set_var_type(&mut self, variable: &Variable, vartype: &Type) {
        let type_store = self.type_store;
        self.emit(vec![
            ZOP::StoreBOperand{array_address: Operand::new_large_const(type_store as i16), index: Operand::new_const(variable.id), operand: Operand::new_const(vartype.clone() as u8)},
        ]);
    }

    /// Copies the variable type of the Operand in `from` to the variable.
    fn copy_var_type(&mut self, variable: &Variable, from: &Operand) {
        let type_store = self.type_store;
        match from {
            &Operand::BoolConst(_) => {
                self.emit(vec![ZOP::SetVarType{variable: variable.clone(), vartype: Type::Bool},]);
                },
            &Operand::StringRef(_) => {
                self.emit(vec![ZOP::SetVarType{variable: variable.clone(), vartype: Type::String},]);
                },
            &Operand::Var(ref var) => {
                self.emit(vec![
                    ZOP::PushVar{variable: variable.clone()},
                    ZOP::GetVarType{variable: var.clone(), result: variable.clone()},
                    ZOP::StoreBOperand{array_address: Operand::new_large_const(type_store as i16), index: Operand::new_const(variable.id), operand: Operand::new_var(variable.id)},
                    ZOP::PullVar{variable: variable.clone()},
                    ]);
                },
            _ => {
                self.emit(vec![ZOP::SetVarType{variable: variable.clone(), vartype: Type::Integer},]);
                },
        };
    }

    /// Stores the variable type of `variable` in the `result` variable.
    fn get_var_type(&mut self, variable: &Variable, result: &Variable) {
        let type_store = self.type_store;
        self.emit(vec![
            ZOP::LoadBOperand{array_address: Operand::new_large_const(type_store as i16), index: Operand::new_const(variable.id), variable: result.clone()},
        ]);
    }

    /// Helper function to add two values according to the types of them and saves type of savevarid
    /// to the global type-store and returns the result.
    pub fn routine_add_types(&mut self) {
        let type_store = self.type_store;
        let val1 = Variable::new(1);  // first argument
        let type1 = Variable::new(2);  // second argument
        let val2 = Variable::new(3);  // third argument
        let type2 = Variable::new(4);  // fourth argument
        let savevarid = Variable::new(5);  // fifth argument
        let result = Variable::new(6);
        let falsestr = self.write_string("false");
        let truestr = self.write_string("true");
        self.emit(vec![
            ZOP::Routine{name: "add_types".to_string(), count_variables: 10},
            ZOP::JE{operand1: Operand::new_var(type1.id), operand2: Operand::new_const(Type::String as u8), jump_to_label: "add_types_resultstring".to_string()},
            ZOP::JE{operand1: Operand::new_var(type2.id), operand2: Operand::new_const(Type::String as u8), jump_to_label: "add_types_resultstring".to_string()},
            ZOP::Add{operand1: Operand::new_var(val1.id), operand2: Operand::new_var(val2.id), save_variable: result.clone()},
            // store type integer for savevarid
            ZOP::StoreBOperand{array_address: Operand::new_large_const(type_store as i16), index: Operand::new_var(savevarid.id), operand: Operand::new_const(Type::Integer as u8)},
            ZOP::Ret{value: Operand::new_var(result.id)},
            ZOP::Label{name: "add_types_resultstring".to_string()},
            // if val1 is string jump to val1isstring
            ZOP::JE{operand1: Operand::new_var(type1.id), operand2: Operand::new_const(Type::String as u8), jump_to_label: "add_types_val1isstring".to_string()},
            // convert val1 to string
            ZOP::JE{operand1: Operand::new_var(type1.id), operand2: Operand::new_const(Type::Bool as u8), jump_to_label: "add_types_val1isbool".to_string()},
            ZOP::Call2S{jump_to_label: "itoa".to_string(), arg: Operand::new_var(val1.id), result: val1.clone()},
            ZOP::Jump{jump_to_label: "add_types_val1isstring".to_string()},
            ZOP::Label{name: "add_types_val1isbool".to_string()},
            ZOP::JE{operand1: Operand::new_var(val1.id), operand2: Operand::new_const(0), jump_to_label: "add_types_val1isfalse".to_string()},
            // set to "true"
            ZOP::StoreVariable{variable: val1.clone(), value: Operand::new_large_const(truestr as i16)},
            ZOP::Jump{jump_to_label: "add_types_val1isstring".to_string()},
            ZOP::Label{name: "add_types_val1isfalse".to_string()},
            ZOP::StoreVariable{variable: val1.clone(), value: Operand::new_large_const(falsestr as i16)},
            ZOP::Label{name: "add_types_val1isstring".to_string()},
            // if val2 is string jump to val2isstring
            ZOP::JE{operand1: Operand::new_var(type2.id), operand2: Operand::new_const(Type::String as u8), jump_to_label: "add_types_val2isstring".to_string()},
            // convert val2 to string
            ZOP::JE{operand1: Operand::new_var(type2.id), operand2: Operand::new_const(Type::Bool as u8), jump_to_label: "add_types_val2isbool".to_string()},
            ZOP::Call2S{jump_to_label: "itoa".to_string(), arg: Operand::new_var(val2.id), result: val2.clone()},
            ZOP::Jump{jump_to_label: "add_types_val2isstring".to_string()},
            ZOP::Label{name: "add_types_val2isbool".to_string()},
            ZOP::JE{operand1: Operand::new_var(val2.id), operand2: Operand::new_const(0), jump_to_label: "add_types_val2isfalse".to_string()},
            // set to "true"
            ZOP::StoreVariable{variable: val2.clone(), value: Operand::new_large_const(truestr as i16)},
            ZOP::Jump{jump_to_label: "add_types_val2isstring".to_string()},
            ZOP::Label{name: "add_types_val2isfalse".to_string()},
            ZOP::StoreVariable{variable: val2.clone(), value: Operand::new_large_const(falsestr as i16)},
            ZOP::Label{name: "add_types_val2isstring".to_string()},
            // add strings
            ZOP::CallVSA2{jump_to_label: "strcat".to_string(), arg1: Operand::new_var(val1.id), arg2: Operand::new_var(val2.id), result: result.clone()},
            // store type string for savevarid
            ZOP::StoreBOperand{array_address: Operand::new_large_const(type_store as i16), index: Operand::new_var(savevarid.id), operand: Operand::new_const(Type::String as u8)},
            ZOP::Ret{value: Operand::new_var(result.id)},
        ]);
    }

    /// Helper function to add two values according to their types.
    fn add_types(&mut self, operand1: &Operand, operand2: &Operand, tmp1: &Variable, tmp2: &Variable, save_variable: &Variable) {
        let type1op = match operand1 {
            &Operand::StringRef(_) => Operand::new_const(Type::String as u8),
            &Operand::BoolConst(_) => Operand::new_const(Type::Bool as u8),
            &Operand::LargeConst(_) => Operand::new_const(Type::Integer as u8),
            &Operand::Const(_) => Operand::new_const(Type::Integer as u8),
            &Operand::Var(ref var) => {
                    self.emit(vec![ZOP::GetVarType{variable: var.clone(), result: tmp1.clone()}]);
                    Operand::new_var(tmp1.id)
                }
        };
        let type2op = match operand2 {
            &Operand::StringRef(_) => Operand::new_const(Type::String as u8),
            &Operand::BoolConst(_) => Operand::new_const(Type::Bool as u8),
            &Operand::LargeConst(_) => Operand::new_const(Type::Integer as u8),
            &Operand::Const(_) => Operand::new_const(Type::Integer as u8),
            &Operand::Var(ref var) => {
                    self.emit(vec![ZOP::GetVarType{variable: var.clone(), result: tmp2.clone()}]);
                    Operand::new_var(tmp2.id)
                }
        };
        self.emit(vec![
            ZOP::CallVS2A5{jump_to_label: "add_types".to_string(),
                arg1: operand1.clone(), arg2: type1op, arg3: operand2.clone(), arg4: type2op, arg5: Operand::new_const(save_variable.id), result: save_variable.clone()},
        ]);
    }

    /// Print one zscii character given as argument.
    pub fn routine_print_char(&mut self) {
        self.emit(vec![
            ZOP::Routine{name: "print_char".to_string(), count_variables: 3},
            ZOP::JL{operand1: Operand::new_var(1), operand2: Operand::new_const(32), jump_to_label: "print_char_?".to_string()},
            ZOP::JG{operand1: Operand::new_var(1), operand2: Operand::new_const(126), jump_to_label: "print_char_?".to_string()},
            ZOP::Jump{jump_to_label: "print_char_normal".to_string()},
            ZOP::Label{name: "print_char_?".to_string()},
            ZOP::StoreVariable{variable: Variable::new(1), value: Operand::new_const('?' as u8)},
            ZOP::Label{name: "print_char_normal".to_string()},
            ZOP::PrintChar{var: Variable::new(1)},
            ZOP::Ret{value: Operand::new_const(0)}
        ]);
    }

    // ================================
    // specific ops

    /// Print strings.
    ///
    /// print is 0OP.
    fn op_print(&mut self, content: &str) {
        let index: usize = self.data.bytes.len();
        self.op_0(0x02);

        let mut text_bytes: Bytes = Bytes{bytes: Vec::new()};
        ztext::encode(&mut text_bytes, content, &self.unicode_table);
        self.data.write_bytes(&text_bytes.bytes, index + 1);
    }

    /// Jumps to a label.
    pub fn op_jump(&mut self, jump_to_label: &str) {
        self.op_1(0x0c, ArgType::LargeConst);
        self.add_jump(jump_to_label.to_string(), JumpType::Jump);
    }



    /// Calls a routine.
    ///
    /// call_1n is 1OP.
    pub fn op_call_1n(&mut self, jump_to_label: &str) {
        self.op_1(0x0f, ArgType::LargeConst);
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);
    }


    /// Calls a routine with an argument(variable) and throws result away
    /// because the value isn't known until all routines are set, it
    /// inserts a pseudo routoune_address.
    ///
    /// call_2n is 2OP.
    pub fn op_call_2n_with_address(&mut self, jump_to_label: &str, address: &str) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, ArgType::LargeConst];
        self.op_2(0x1a, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        // the address of the argument
        self.add_jump(address.to_string(), JumpType::Routine);
    }

    /// Calls a routine with one argument an throws result away.
    ///
    /// call_2n is 2OP.
    pub fn op_call_2n_with_arg(&mut self, jump_to_label: &str, arg: &Operand) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, op::arg_type(&arg)];
        self.op_2(0x1a, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        op::write_argument(arg, &mut self.data.bytes);  // just one argument
    }

    /// Calls a routine with one argument and stores return value in result.
    ///
    /// call_2s is 2OP.
    pub fn op_call_2s(&mut self, jump_to_label: &str, arg: &Operand, result: &Variable) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, op::arg_type(&arg), ArgType::Variable];
        self.op_2(0x19, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        op::write_argument(arg, &mut self.data.bytes);  // just one argument
        self.data.append_byte(result.id);
    }

    /// Calls a routine with two arguments and throws result away.
    ///
    /// call_vn is VAROP.
    pub fn op_call_vn_a2(&mut self, jump_to_label: &str, arg1: &Operand, arg2: &Operand) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, op::arg_type(&arg1), op::arg_type(&arg2), ArgType::Nothing];
        self.op_var(0x19, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        op::write_argument(arg1, &mut self.data.bytes);
        op::write_argument(arg2, &mut self.data.bytes);
    }

    /// Calls a routine with three arguments and throws result away.
    ///
    /// call_vn is VAROP.
    pub fn op_call_vn_a3(&mut self, jump_to_label: &str, arg1: &Operand, arg2: &Operand, arg3: &Operand) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, op::arg_type(&arg1), op::arg_type(&arg2), op::arg_type(&arg3)];
        self.op_var(0x19, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        op::write_argument(arg1, &mut self.data.bytes);
        op::write_argument(arg2, &mut self.data.bytes);
        op::write_argument(arg3, &mut self.data.bytes);
    }

    /// Calls a routine with two arguments and stores return value in result.
    ///
    /// call_vs is VAROP.
    pub fn op_call_vs_a2(&mut self, jump_to_label: &str, arg1: &Operand, arg2: &Operand, result: &Variable) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, op::arg_type(&arg1), op::arg_type(&arg2), ArgType::Nothing];
        self.op_var(0x0, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        op::write_argument(arg1, &mut self.data.bytes);
        op::write_argument(arg2, &mut self.data.bytes);
        self.data.append_byte(result.id);
    }

    /// Calls a routine with three arguments and stores return value in result.
    ///
    /// call_vs is VAROP.
    pub fn op_call_vs_a3(&mut self, jump_to_label: &str, arg1: &Operand, arg2: &Operand, arg3: &Operand, result: &Variable) {
        let args: Vec<ArgType> = vec![ArgType::LargeConst, op::arg_type(&arg1), op::arg_type(&arg2), op::arg_type(&arg3)];
        self.op_var(0x0, args);

        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        op::write_argument(arg1, &mut self.data.bytes);
        op::write_argument(arg2, &mut self.data.bytes);
        op::write_argument(arg3, &mut self.data.bytes);
        self.data.append_byte(result.id);
    }

    /// Calls a routine with five arguments and stores the return value.
    ///
    /// call_vs2 is VAROP with additional types-byte.
    pub fn op_call_vs2_a5(&mut self, jump_to_label: &str, arg1: &Operand, arg2: &Operand, arg3: &Operand, arg4: &Operand, arg5: &Operand, result: &Variable) {
        let args1: Vec<ArgType> = vec![ArgType::LargeConst, op::arg_type(&arg1), op::arg_type(&arg2), op::arg_type(&arg3)];
        let args2: Vec<ArgType> = vec![op::arg_type(&arg4), op::arg_type(&arg5), ArgType::Nothing, ArgType::Nothing];
        self.op_var(0xC, args1);
        self.data.append_byte(op::encode_variable_arguments(args2));
        // the address of the jump_to_label
        self.add_jump(jump_to_label.to_string(), JumpType::Routine);

        op::write_argument(arg1, &mut self.data.bytes);
        op::write_argument(arg2, &mut self.data.bytes);
        op::write_argument(arg3, &mut self.data.bytes);
        op::write_argument(arg4, &mut self.data.bytes);
        op::write_argument(arg5, &mut self.data.bytes);
        self.data.append_byte(result.id);
    }

    /// Jumps to a label if the value of operand1 is equal to operand2.
    pub fn op_je(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![op::arg_type(operand1), op::arg_type(operand2)];
        self.op_2(0x01, args);

        op::write_argument(operand1, &mut self.data.bytes);
        op::write_argument(operand2, &mut self.data.bytes);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::Branch);
    }

    /// Jumps to a label if the value of operand1 is not equal to operand2.
    pub fn op_jne(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {
        self.emit(vec![
            ZOP::JL{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()},
            ZOP::JG{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()}
            ]);
    }

    /// Jumps to a label if the value of operand1 is lower than operand2 (compared as i16).
    pub fn op_jl(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![op::arg_type(operand1), op::arg_type(operand2)];
        self.op_2(0x02, args);

        op::write_argument(operand1, &mut self.data.bytes);
        op::write_argument(operand2, &mut self.data.bytes);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::Branch);
    }

    /// Jumps to a label if the value of operand1 is lower than or equal operand2 (compared as i16).
    pub fn op_jle(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {
        self.emit(vec![
            ZOP::JL{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()},
            ZOP::JE{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()}
            ]);
    }

    /// Jumps to a label if the value of operand1 is greater than or equal operand2 (compared as i16).
    pub fn op_jge(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {
        self.emit(vec![
            ZOP::JG{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()},
            ZOP::JE{operand1: operand1.clone(), operand2: operand2.clone(), jump_to_label: jump_to_label.to_string()}
            ]);
    }

    /// Jumps to a label if the value of operand1 is greater than operand2.
    pub fn op_jg(&mut self, operand1: &Operand, operand2: &Operand, jump_to_label: &str) {

        let args: Vec<ArgType> = vec![op::arg_type(operand1), op::arg_type(operand2)];
        self.op_2(0x03, args);

        op::write_argument(operand1, &mut self.data.bytes);
        op::write_argument(operand2, &mut self.data.bytes);

        // jump
        self.add_jump(jump_to_label.to_string(), JumpType::Branch);
    }

    /// Reads keys from the keyboard and saves the asci-value in local_var_id.
    ///
    /// read_char is VAROP.
    pub fn op_read_char_timer(&mut self, local_var_id: u8, timer: u8, routine: &str) {
        let args: Vec<ArgType> = vec![ArgType::SmallConst, ArgType::SmallConst, ArgType::LargeConst, ArgType::Nothing];
        self.op_var(0x16, args);

        // write argument value
        self.data.append_byte(0x01);

        // write timer
        self.data.append_byte(timer);

        // writes routine
        self.add_jump(routine.to_string(), JumpType::Routine);

        // write varible id
        self.data.append_byte(local_var_id);
    }


    /// Prints a unicode char to the current stream.
    pub fn op_print_unicode_char(&mut self, character: u16) {

        self.op_1(0xbe, ArgType::SmallConst);
        self.data.append_byte(0x0b);
        // 0x00 means LargeConst, then 0x03 means omitted, 0x02 means Variable, 0x01 means SmallConst
        let byte = 0x00 << 6 | 0x03 << 4 | 0x03 << 2 | 0x03 << 0;
        self.data.append_byte(byte);
        self.data.append_u16(character);
    }

    /// Prints variable with unicode char to the current stream.
    pub fn op_print_unicode_var(&mut self, variable: &Variable) {

        self.op_1(0xbe, ArgType::SmallConst);
        self.data.append_byte(0x0b);
        // 0x00 means LargeConst, then 0x03 means omitted, 0x02 means Variable, 0x01 means SmallConst
        let byte = 0x02 << 6 | 0x03 << 4 | 0x03 << 2 | 0x03 << 0;
        self.data.append_byte(byte);
        self.data.append_byte(variable.id);
    }

    /// Prints a unicode string to the current output stream.
    pub fn op_print_unicode_str(&mut self, address: &Operand) {
        self.emit(vec![ZOP::Call2NWithArg{jump_to_label: "print_unicode".to_string(), arg: address.clone()}]);
    }

    /// Prints a ZSCII character.
    pub fn op_print_char(&mut self, variable: &Variable) {
        let args: Vec<ArgType> = vec![ArgType::Variable, ArgType::Nothing, ArgType::Nothing, ArgType::Nothing];
        self.op_var(0x5, args);
        self.data.append_byte(variable.id);
    }

    // ================================
    // general ops

    /// Binary representation for op-codes with 0 operators.
    fn op_0(&mut self, value: u8) {
        self.data.append_bytes(&op::op_0(value));
    }

    /// Binary representation for op-codes with 1 operator.
    fn op_1(&mut self, value: u8, arg_type: ArgType) {
        self.data.append_bytes(&op::op_1(value, arg_type));
    }

    /// Binary representation for op-codes with 1 operator.
    fn op_2(&mut self, value: u8, arg_types: Vec<ArgType>) {
        self.data.append_bytes(&op::op_2(value, arg_types));
    }

    /// Binary representation for a variable.
    fn op_var(&mut self, value: u8, arg_types: Vec<ArgType>) {
        self.data.append_bytes(&op::op_var(value, arg_types));
    }
}

/// Align the address to the given align-parameter.
fn align_address(address: u32, align: u32) -> u32 {
    address + (align - (address % align)) % align
}

/// Returns the routine address, should be `adress % 8 == 0` (because its a packed address).
fn routine_address(address: u32) -> u32 {
    return align_address(address, 8);
}

// ================================
// Test functions

#[cfg(test)]
mod tests {
    use super::{routine_address, align_address};
    use super::*;

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

        zfile.op_jump("labelname");
        assert_eq!(zfile.data.len(), 3);

        zfile.label("labelname");
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
        let (labels, jumps1, bytes1) =  zfile.write_zop(&ZOP::Label{name: "Start".to_string()}, true);
        assert_eq!(jumps1.len() + bytes1.len(), 0);
        assert_eq!(labels.len(), 1);
        let (labels2, jumps, bytes) =  zfile.write_zop(&ZOP::Jump{jump_to_label: "Start".to_string()}, true);
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
        assert_eq!(op::op_read_char(0x01),vec![0xF6,0x7F,0x01,0x01]);
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
        assert_eq!(op::op_ret(&Operand::new_large_const(0x0101 as i16)),vec![0x8B,0x01,0x01]);
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

    #[test]
    fn test_op_not() {
            assert_eq!(op::op_not(&Operand::new_var(1),&Variable::new(2)),vec![0xf8,0xbf,0x01,0x02]);
    }

    #[test]
    fn test_op_get_cursor() {
            assert_eq!(op::op_get_cursor(&Operand::new_var(1)),vec![0xf0,0xbf,0x01]);
    }

    #[test]
    fn test_op_set_cursor_operand() {
            assert_eq!(op::op_set_cursor_operand(&Operand::new_var(1), &Operand::new_var(2)),vec![0xef,0xaf,0x01,0x02]);
    }

    #[test]
    fn test_op_erase_line() {
            assert_eq!(op::op_erase_line(),vec![0xee,0x7f,0x01]);
    }
}
