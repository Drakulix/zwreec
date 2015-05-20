//! The `codegen` module is for the creating of zcode from an ast

use frontend::ast;
use super::super::backend::zcode::zfile;
use utils::file;

pub fn generate_zcode(ast: ast::AST, output_file_name: &str) {

    let mut codegenerator = Codegen::new(ast);
    codegenerator.start_codegen();
    file::save_bytes_to_file(output_file_name, &(*codegenerator.zfile_bytes()) );
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
        self.zfile.op_call_1n("main");
        self.zfile.op_quit();
        self.zfile.routine("main", 0);

        self.ast.to_zcode(&mut self.zfile);

        self.zfile.op_quit();

        self.zfile.end();
    }

    pub fn zfile_bytes(&self) -> &Vec<u8> {
        &self.zfile.data.bytes
    }
}

#[test]
fn it_works() {

}
