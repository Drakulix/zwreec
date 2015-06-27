pub fn main() {
    extern crate syntex;
    extern crate rustlex_codegen;
    use std::env;
    use std::path::Path;

    let mut registry = syntex::Registry::new();
    rustlex_codegen::plugin_registrar(&mut registry);
    let src = Path::new("src/zwreec/frontend/rustlex.in.rs");
    let dst = Path::new(&env::var_os("OUT_DIR").unwrap()).join("rustlex.rs");
    registry.expand("rustlex", &src, &dst).unwrap();
}
