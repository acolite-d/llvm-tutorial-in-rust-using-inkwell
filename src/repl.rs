use std::io::Write;

use crate::{
    cli::Cli,
    frontend::{
        lexer::{Lex, Token},
        parser::{parse_definition, parse_extern, parse_top_level_expr},
    },
    backend::llvm_backend::{LLVMCodeGen, LLVMContext}
};

// I have two different kinds of Read-Print-Eval-Loops here. One simply runs
// frontend of Kaleidoscope, producing AST, printing debug representation of that.
// The other does the additional step of generating LLVM IR, JIT compiling and running it.
#[allow(unused)]
pub fn ast_parser_driver() {
    let mut input_buf = String::new();

    loop {
        print!("Ready >> ");
        std::io::stdout().flush().unwrap();
        let _ = std::io::stdin().read_line(&mut input_buf);

        let mut tokens = input_buf.lex().peekable();

        match tokens.peek() {
            None => continue,

            Some(Token::FuncDef) => match parse_definition(&mut tokens) {
                Ok(ast) => {
                    println!("Parsed a function definition.");
                    dbg!(ast);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    _ = tokens.next();
                }
            },

            Some(Token::Extern) => match parse_extern(&mut tokens) {
                Ok(ast) => {
                    println!("Parsed an extern.");
                    dbg!(ast);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    _ = tokens.next();
                }
            },

            Some(Token::Semicolon) => {
                _ = tokens.next();
            }

            Some(_top_level_token) => match parse_top_level_expr(&mut tokens) {
                Ok(ast) => {
                    println!("Parsed a top-level expression.");
                    dbg!(ast);
                }
                Err(err) => {
                    eprintln!("Error on top-level: {}", err);
                    _ = tokens.next();
                }
            },
        }

        std::mem::drop(tokens);
        input_buf.clear();
    }
}

pub fn driver(cli_args: &Cli) {
    let context = inkwell::context::Context::create();

    let sesh_ctx = LLVMContext::new(&context, cli_args.opt_level);
    let mut input_buf = String::new();

    loop {
        print!("Ready >> ");
        std::io::stdout().flush().unwrap();
        let _ = std::io::stdin().read_line(&mut input_buf);

        let mut tokens = input_buf.lex().peekable();

        match tokens.peek() {
            None => continue,

            Some(Token::FuncDef) => match parse_definition(&mut tokens) {
                Ok(ast) => {
                    match ast.codegen(&sesh_ctx) {
                        Ok(_ir) => {
                            sesh_ctx.run_passes(&cli_args.passes);

                            cli_args.inspect_tree_p
                                .then(|| println!("Abstract Syntax Tree Representation:\n{:#?}\n", &ast));
                            cli_args.inspect_ir_p
                                .then(|| sesh_ctx.dump_module());
                            cli_args.inspect_asm_p
                                .then(|| sesh_ctx.dump_assembly());
                        }
                        Err(e) => eprintln!("Backend error: {}", e),
                    }
                }
                Err(err) => {
                    eprintln!("Frontend Error: {}", err);
                    _ = tokens.next();
                }
            },

            Some(Token::Extern) => match parse_extern(&mut tokens) {
                Ok(ast) => {
                    match ast.codegen(&sesh_ctx) {
                        Ok(_ir) => {
                            cli_args.inspect_tree_p
                                .then(|| println!("Abstract Syntax Tree Representation:\n{:#?}\n", &ast));
                            cli_args.inspect_ir_p
                                .then(|| sesh_ctx.dump_module());
                            cli_args.inspect_asm_p
                                .then(|| sesh_ctx.dump_assembly());
                        }
                        Err(e) => eprintln!("Backend error: {}", e),
                    }
                }
                Err(err) => {
                    eprintln!("Frontend Error: {}", err);
                    _ = tokens.next();
                }
            },

            Some(Token::Semicolon) => {
                _ = tokens.next();
            }

            Some(_top_level_token) => match parse_top_level_expr(&mut tokens) {
                Ok(ast) => {
                    match ast.codegen(&sesh_ctx) {
                        Ok(_ir) => {
                            sesh_ctx.run_passes(&cli_args.passes);

                            cli_args.inspect_tree_p
                                .then(|| println!("Abstract Syntax Tree Representation:\n{:#?}\n", &ast));
                            cli_args.inspect_ir_p
                                .then(|| sesh_ctx.dump_module());
                            cli_args.inspect_asm_p
                                .then(|| sesh_ctx.dump_assembly());

                            unsafe {
                                let res = sesh_ctx
                                    .jit_eval()
                                    .expect("Failed to JIT top level expression into function!");

                                println!("Jit compiled and evaluated to: {res}");
                            }
                        }
                        Err(e) => eprintln!("Backend error: {}", e),
                    }

                    sesh_ctx.delete_top_level_expr();
                }
                Err(err) => {
                    eprintln!("Frontend Error: {}", err);
                    _ = tokens.next();
                }
            },
        }

        std::mem::drop(tokens);
        input_buf.clear();
    }
}
