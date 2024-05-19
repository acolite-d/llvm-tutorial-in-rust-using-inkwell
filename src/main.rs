#[macro_use]
extern crate lazy_static;

mod backend;
mod frontend;
mod repl;

use inkwell::targets;

use backend::llvm_backend;
use repl::{ast_parser_driver, llvm_ir_gen_driver};

fn main() {
    let target_config = targets::InitializationConfig::default();

    targets::Target::initialize_native(&target_config)
        .expect("Failed to initialize native machine target!");

    

    llvm_ir_gen_driver();
    // ast_parser_driver();
}
