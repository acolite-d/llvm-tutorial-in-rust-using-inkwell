#[macro_use]
extern crate lazy_static;

mod backend;
mod frontend;
mod repl;

use backend::llvm_backend;
use repl::{ast_parser_driver, llvm_ir_gen_driver};

fn main() {
    llvm_ir_gen_driver();
}
