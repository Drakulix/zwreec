#![doc(html_root_url="https://drakulix.github.io/zwreec/",
    html_logo_url="https://dl.dropboxusercontent.com/u/70410095/zwreec/logo.png")]
//! Twee-to-Zcode compile library.
//!
//! Zwreec is a compiler for [interactive fiction](http://en.wikipedia.org/wiki/Interactive_fiction)
//! in the Twee format (created by the [Twine software](http://en.wikipedia.org/wiki/Twine_(software)))
//! to [Z-Machine](http://en.wikipedia.org/wiki/Z-machine) instructions (Zcode)
//! which can be run with interpreters like [frotz](http://frotz.sourceforge.net).
//!
//! This library was developed as part of the course "Softwareprojekt
//! Übersetzerbau" in 2015.
//!
//! # Requirements and Usage
//!
//! The library uses a [fork of rustlex](https://github.com/Drakulix/rustlex)
//! to do lexical analysis.
//! You can install Rust stable or nightly as you like via the
//! [provided binaries](http://www.rust-lang.org/install.html)
//! or the rustup script:
//!
//! ```sh
//! $ curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2015-06-02
//! ```
//!
//! Or install the Ubuntu/Debian Packages for
//! [Rust](http://ppa.launchpad.net/hansjorg/rust/ubuntu/pool/main/r/rust-nightly/) and
//! [Cargo](http://ppa.launchpad.net/hansjorg/rust/ubuntu/pool/main/c/cargo-nightly/).
//!
//! To use zwreec in your project you can add it as a dependency to your `Cargo.toml`.
//!
//! ```toml
//! [dependencies.zwreec]
//! git = "https:://github.com/Drakulix/zwreec"
//! ```
//!
//! Then you can use it in your crate root.
//!
//! ```rust
//! extern crate zwreec;
//! ```
//!
//! # Features
//!
//! Only rudimentary Twee features are supported right now. This is about to change in the upcoming weeks.
//! Check the github issues for more information on the currently supported features.
//!
//! # Example
//!
//! The following example is itself a full working command line compiler. It is a simpler
//! version of the [Reference Binary Implementation](#reference-binary-implementation).
//!
//! ```no_run
//! extern crate zwreec;
//!
//! use std::env;
//! use std::error::Error;
//! use std::fs::File;
//! use std::path::Path;
//!
//! fn main() {
//!     let mut args: Vec<String> = env::args().collect();
//!
//!     if args.len() != 2 { panic!("Need exactly one input file!"); }
//!
//!     let cfg = zwreec::config::Config::default_config();
//!     let mut input = match File::open(Path::new(&args[1])) {
//!         Ok(file) => file,
//!         Err(why) => { panic!("Couldn't open input: {}", Error::description(&why)); }
//!     };
//!     let mut output = match File::create(Path::new("a.z8")) {
//!         Ok(file) => file,
//!         Err(why) => { panic!("Couldn't open output: {}", Error::description(&why)); }
//!     };
//!
//!     zwreec::compile(cfg, &mut input, &mut output);
//! }
//! ```
//!
//! # Reference Binary Implementation
//!
//! Zwreecs [Github-Repository](https://github.com/Drakulix/zwreec) contains a
//! reference binary implementation that uses this library and provides a simple
//! command line interface to compile Twee files.
//!
//! To build the binary, you need the rust version as outlined in [Requirements and
//! Usage](#requirements-and-usage). Then you can build the binary using
//!
//! ```sh
//! $ cargo build
//! ```
//!
//! The resulting binary can be found at `target/debug/zwreec`.
//!
//! # Logging
//!
//! Zwreec uses the logging facade provided by
//! [log](../log/index.html). The reference binary implementation of zwreec also includes an
//! implementation of the `Log` trait.

extern crate rustlex_codegen as rustlex;
#[macro_use] extern crate log;
extern crate time;
extern crate term;
extern crate getopts;

#[macro_use] pub mod utils;
pub mod config;
pub mod frontend;
pub mod backend;

use config::{Config,TestCase};
use std::error::Error;
use std::io::{BufReader,Cursor,Read,Write};


/// Compiles a Twee Input to Zcode
///
/// This is the main call into the Zwreec compiler. It will process `input: Read`
/// by calling the different parts of the compiler-chain, notably `frontend::lexer`
/// to generate a tokenstream, `frontend::parser` and `frontend::ast` to generate
/// the Abstract Syntax Tree and lastly `frontend::codegen` to generate the Zcode.
///
/// # Example
///
/// ```no_run
/// # use std::env;
/// # use std::fs::File;
/// # use std::path::Path;
/// # let mut args: Vec<String> = env::args().collect();
/// #
/// # if args.len() != 2 { panic!("Need exactly one input file!"); }
/// #
/// let cfg = zwreec::config::Config::default_config();
/// let mut input = File::open(Path::new(&args[1])).unwrap();
/// let mut output = File::create(Path::new("a.z8")).unwrap();
///
/// zwreec::compile(cfg, &mut input, &mut output);
/// ```
#[allow(unused_variables)]
pub fn compile<R: Read, W: Write>(cfg: Config, input: &mut R, output: &mut W) {

    // check the data if it has a bom
    let mut cursor = handle_bom_encoding(input);

    // tokenize
    let tokens = frontend::lexer::lex(&cfg, &mut cursor);

    // create parser
    let parser = frontend::parser::Parser::new(&cfg);

    // create ast builder
    let ast_builder = frontend::ast::ASTBuilder::new(&cfg);

    // build up ast from tokens
    let ast = ast_builder.build(parser.parse(tokens.inspect(|ref token| {
        debug!("{:?}", token);
    })));
    debug!("{:?}", ast);

    // create code
    frontend::codegen::generate_zcode(&cfg, ast, output);
}

/// checks the input if there is an bom, if true it will delete it
fn handle_bom_encoding<'a, R: Read>(input: &'a mut R) -> Cursor<Vec<u8>> {
    let mut reader = BufReader::new(input);
    let mut content = String::new();
    match reader.read_to_string(&mut content) {
        Err(why) => error!("Couldn't read {}", Error::description(&why)),
        Ok(_) => (),
    };

    let mut v: Vec<u8> = content.bytes().collect();
    if v.len() < 5 {
        error!("The file is to short for a valid twee File");
    }
    let has_bom = if &v[0..3] == [0xef, 0xbb, 0xbf] {
        true
    } else {
        false
    };
    if has_bom {
        debug!("File has Byte order mark");
        v.remove(0);
        v.remove(0);
        v.remove(0);
    }

    let cursor: Cursor<Vec<u8>> = Cursor::new(v);

    cursor
}

/// Run internal library tests.
///
/// This function is used to circumvent certain parts of the compiler toolchain.
/// It currently only processes `TestCase::ZcodeBackend` which creates a Zcode
/// file using all available OP-Codes.
///
/// **Warning:** This function should be considered unstable and might be removed
/// in later versions.
///
/// # Example
///
/// ```
/// # use std::env;
/// # use std::fs::File;
/// # use std::path::Path;
/// let mut cfg = zwreec::config::Config::default_config();
/// cfg.test_cases.push(zwreec::config::TestCase::ZcodeBackend);
///
/// let mut input: Option<File> = None;
/// let mut output = Some(File::create(Path::new("a.z8")).unwrap());
///
/// zwreec::test_library(cfg, &mut input, &mut output);
/// ```
#[allow(unused_variables)]
pub fn test_library<R: Read, W: Write>(cfg: Config, input: &mut Option<R>, output: &mut Option<W>) {
    for case in cfg.test_cases {
        match case {
            TestCase::ZcodeBackend => {
                match output.as_mut() {
                     Some(o) => backend::zcode::temp_create_zcode_example(o),
                     None => error!("TestCase::ZcodeBackend requires output!"),
                }
            }
        }
    }
}
