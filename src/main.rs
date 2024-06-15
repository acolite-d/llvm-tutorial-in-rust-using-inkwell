#[macro_use]
extern crate lazy_static;

mod backend;
mod frontend;
mod repl;
mod cli;

use clap::{Parser};
use inkwell::targets;

use cli::{Cli, OptLevel};
use repl::{ast_parser_driver, llvm_ir_gen_driver};

fn main() {

    let cli = Cli::parse();

    let target_config = targets::InitializationConfig::default();

    targets::Target::initialize_native(&target_config)
        .expect("Failed to initialize native machine target!");

    targets::Target::initialize_all(&target_config);

    if cli.use_frontend_only {
        ast_parser_driver();
    } else {
        llvm_ir_gen_driver(cli.opt_level, &cli.passes);
    }
}
