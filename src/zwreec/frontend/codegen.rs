//! The `codegen` module is for the creating of zcode from an ast

use std::error::Error;
use std::io::Write;
use frontend::ast;
use super::super::backend::zcode::zfile;

pub fn generate_zcode<W: Write>(ast: ast::AST, output: &mut W) {
    let mut codegenerator = Codegen::new(ast);
    codegenerator.start_codegen();
    match output.write_all(&(*codegenerator.zfile_bytes())) {
        Err(why) => {
            panic!("Could not write to output: {}", Error::description(&why));
        },
        Ok(_) => {
            info!("Wrote zcode to output");
        }
    };
}

struct Codegen {
    ast: ast::AST,
    zfile: zfile::Zfile
}

impl Codegen {
    pub fn new(ast: ast::AST) -> Codegen {
        Codegen {
            ast: ast,
            zfile: zfile::Zfile::new()
        }
    }

    pub fn start_codegen(&mut self) {
        self.zfile.start();
        self.zfile.op_call_1n("Start");
        //self.zfile.op_quit();
        //self.zfile.routine("main", 0);

        self.ast.to_zcode(&mut self.zfile);
        
        self.zfile.op_quit();

        self.zfile.end();
    }

    pub fn zfile_bytes(&self) -> &Vec<u8> {
        &self.zfile.data.bytes
    }
}
