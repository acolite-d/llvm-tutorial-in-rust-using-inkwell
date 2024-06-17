#[macro_use]
extern crate lazy_static;

mod backend;
mod cli;
mod frontend;
mod repl;

use clap::Parser;
use inkwell::targets;

fn main() {
    let cli = cli::Cli::parse();

    let target_config = targets::InitializationConfig::default();

    targets::Target::initialize_native(&target_config)
        .expect("Failed to initialize native machine target!");

    targets::Target::initialize_all(&target_config);

    // start REPL drivers, infinite loops
    if cli.use_frontend_only {
        repl::ast_parser_driver();
    } else {
        repl::llvm_ir_gen_driver(cli.opt_level, &cli.passes);
    }
}
