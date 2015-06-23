//! Processes Twee files and builds an abstract syntax tree.
//!
//! The Frontend is responsible for processing a Twee file by
//! generating a Token Stream from the input and parsing this stream to build
//! an Abstract Syntax Tree.
//!
//! # Example
//!
//! This small example shows how the different submodules interact and can 
//! be used to parse a Twee-File.
//!
//! ```
//! # extern crate zwreec;
//! use std::io::Cursor;
//!
//! let cfg = zwreec::config::Config::default_config();
//! let mut twee = Cursor::new("::Start\nHello World".to_string().into_bytes());
//!
//! // Generate Token Stream
//! let tokens = zwreec::frontend::lexer::lex(&cfg, &mut twee);
//!
//! // Parse Tokens
//! let p = zwreec::frontend::parser::Parser::new(&cfg);
//! let ast = zwreec::frontend::ast::AST::build(
//!     p.parse(tokens)
//! );
//! ```
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod codegen;
pub mod expressionparser;
