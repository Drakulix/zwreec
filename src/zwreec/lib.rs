#![doc(html_root_url="https://drakulix.github.io/zwreec/",
    html_logo_url="https://dl.dropboxusercontent.com/u/70410095/zwreec/logo.png")]
//! Twee-to-Zcode compile library
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
//! The library uses a [fork of rustlex](https://github.com/Farthen/rustlex) to
//! to do lexical analysis. Since rustlex requires the Rust Nightly Builds, so 
//! does zwreec. More precisely, this library was *only* developed and tested 
//! against Rust 1.2.0-nightly, build 2015-06-01. You can install this specific 
//! version using:
//!
//! ```sh
//! $ curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2015-06-02
//! ```
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
//! To build the binary, you need the rust version as outlined in [Requirements and
//! Usage](#requirements-and-usage). Then you can build the binary using
//!
//! ```sh
//! $ cargo build --release
//! ```
//! 
//! The resulting binary can be found at `target/release/zwreec`. 
//!
//! # Logging
//!
//! Zwreec uses the logging facade provided by
//! [log](../log/index.html). The reference binary implementation of zwreec also includes an
//! implementation of the `Log` trait.

#![feature(plugin)]
#![plugin(rustlex)]
#[allow(plugin_as_library)]
extern crate rustlex;
#[macro_use] extern crate log;
extern crate time;
extern crate term;
extern crate getopts;

pub mod config;
pub mod frontend;
pub mod backend;
pub use backend::zcode::zfile;
pub mod utils;

use frontend::codegen;

use config::{Config,TestCase};
use std::io::{Read,Write};


#[allow(unused_variables)]
pub fn compile<R: Read, W: Write>(cfg: Config, input: &mut R, output: &mut W) {
    // tokenize
    let tokens = frontend::lexer::lex(&cfg, input);

    //create parser
    let parser = frontend::parser::Parser::new(&cfg);

    //build up ast from tokens
    let ast = frontend::ast::AST::build(parser.parse(tokens.inspect(|ref token| {
        debug!("{:?}", token);
    })));
    ast.print(false);

    // create code
    codegen::generate_zcode(&cfg, ast, output);
}

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
