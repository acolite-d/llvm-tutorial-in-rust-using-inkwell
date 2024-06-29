use std::fs::read_to_string;
use std::process::exit;

#[macro_use]
extern crate lazy_static;

mod backend;
mod cli;
mod frontend;
mod repl;
mod compile;

use clap::Parser;
use inkwell::targets;

fn main() {
    let cli = cli::Cli::parse();

    let target_config = targets::InitializationConfig::default();

    targets::Target::initialize_native(&target_config)
        .expect("Failed to initialize native machine target!");

    targets::Target::initialize_all(&target_config);

    // If a positional argument of file was passed, then the program runs in compile mode,
    // taking that file and compiling it to an object/assembly file
    if let Some(ref file_path) = cli.file {
        match read_to_string(file_path) {
            Ok(src_code) => {
                compile::compile_src(&src_code, &cli)
                    .expect("Failed to compile to object");
                exit(0);
            }
            Err(_) => {
                eprintln!("File not found, please make sure it exists!");
                exit(-1);
            }
        }
    }

    // If no positional arguments, start REPL drivers, infinite loops
    if cli.inspect_tree {
        repl::ast_parser_driver();
    } else {
        repl::llvm_ir_gen_driver(cli.opt_level, &cli.passes);
    }
}
