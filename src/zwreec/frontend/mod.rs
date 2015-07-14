//! Processes Twee files and builds an abstract syntax tree.
//!
//! The frontend is responsible for processing a Twee file by
//! generating a Token Stream from the input and parsing this stream to build
//! an Abstract Syntax Tree.
//!
//! The first stage in generating the AST is the [lexer](lexer/index.html). The tokens generated
//! by the lexer are then fed to the [parser](parser/index.html). The parser generates a
//! sequence of operations that are then used to generate the [AST](ast/index.html). The AST can
//! then be parsed to generate code in the [backend](../backend/index.html).
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
//! // Clean Input
//! let mut cursor = zwreec::frontend::screener::handle_bom_encoding(&mut twee);
//!
//! // Generate Token Stream
//! let tokens = zwreec::frontend::lexer::lex(cfg.clone(), &mut cursor);
//!
//! // Parse Tokens
//! let p = zwreec::frontend::parser::Parser::new(cfg.clone());
//! let ast: Vec<zwreec::frontend::ast::ASTNode> = zwreec::frontend::ast::ASTBuilder::build(cfg, p.parse(tokens)).collect();
//! ```

pub mod ast;
pub mod evaluate_expression;
pub mod expressionparser;
pub mod lexer;
pub mod parser;
pub mod screener;
