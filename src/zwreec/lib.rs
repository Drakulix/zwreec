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
//! To build Zwreec from Source, you will need to have both Rust 1.1.0 and Cargo installed on your
//! system. You can download the Rust binarys on their
//! [website](http://www.rust-lang.org/install.html), by using your system's package manager or by
//! running this in your shell:
//!
//! ```sh
//! $ curl -sSf https://static.rust-lang.org/rustup.sh | sh
//! ```
//!
//! Cargo should be installed alongside current Rust binaries.
//!
//! The library uses a [fork of rustlex](https://github.com/Drakulix/rustlex)
//! to do lexical analysis.
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
//! To build the binary, you need the Rust version as outlined in [Requirements and
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
//! [log](../log/index.html). The reference binary implementation of Zwreec also includes an
//! implementation of the `Log` trait.

// Warn if documentation is missing
#![warn(missing_docs)]

extern crate rustlex_codegen as rustlex;
#[macro_use] extern crate log;
extern crate getopts;
extern crate term;
extern crate time;

#[macro_use] pub mod utils;
pub mod backend;
pub mod config;
pub mod frontend;

use config::{Config,TestCase};
use std::io::{Read,Write};
use utils::extensions::cached;


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
    let cursor = frontend::screener::handle_bom_encoding(input);

    // tokenize
    let cfg_tokens = cfg.clone();
    let (tokens, join_tokens) = cached(move || {
        frontend::lexer::lex(cfg_tokens, cursor)
    });

    // create parser
    let cfg_parser = cfg.clone();
    let (ast_ops, join_ops) = cached(move || {
        frontend::parser::Parser::new(cfg_parser).parse(
            tokens.inspect(|ref token| {
                debug!("{:?}", token);
            })
        )
    });

    // build up ast from tokens
    let cfg_ast = cfg.clone();
    let (ast, join_ast) = cached( move || {
        frontend::ast::ASTBuilder::build(cfg_ast, ast_ops)
    });

    // create code
    backend::codegen::generate_zcode(&cfg, ast.inspect(|ref passage| {
        debug!("{:?}", passage);
    }), output);

    match join_tokens.join() {
        Err(x) => panic!(x),
        _ => {}
    }

    match join_ops.join() {
        Err(x) => panic!(x),
        _ => {}
    }

    match join_ast.join() {
        Err(x) => panic!(x),
        _ => {}
    }
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
