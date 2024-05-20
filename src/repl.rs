use std::io::Write;

use inkwell::values::AnyValue;

use crate::frontend::{
    ast::AST,
    lexer::{Token, Lex},
    parser::{
        parse_extern, 
        parse_definition, 
        parse_prototype, 
        parse_top_level_expr,
    }
};

use crate::backend::llvm_backend::{LLVMCodeGen, LLVMContext};


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

pub fn llvm_ir_gen_driver() {
    let context = inkwell::context::Context::create();

    let sesh_ctx = LLVMContext::new(&context);
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
                    match ast.codegen(&sesh_ctx) {
                        Ok(ir) => {
                            sesh_ctx.run_passes();
                            sesh_ctx.dump_module();
                        },
                        Err(e) => eprintln!("Backend error: {}", e)
                    }
                }
                Err(err) => {
                    eprintln!("Frontend Error: {}", err);
                    _ = tokens.next();
                }
            },

            Some(Token::Extern) => match parse_extern(&mut tokens) {
                Ok(ast) => {
                    println!("Parsed an extern.");
                    match ast.codegen(&sesh_ctx) {
                        Ok(ir) => sesh_ctx.dump_module(),
                        Err(e) => eprintln!("Backend error: {}", e)
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
                    println!("Parsed a top level expression.");
                    match ast.codegen(&sesh_ctx) {
                        Ok(ir) => { 
                            sesh_ctx.run_passes();
                            sesh_ctx.dump_module();
                            
                            unsafe {
                                let res = sesh_ctx.jit_eval()
                                    .expect("Failed to JIT top level pression into function!");

                                println!("Jit compiled and evaluated to: {res}");

                                ir.into_function_value().delete(); 
                            }
                        }
                        Err(e) => eprintln!("Backend error: {}", e)
                    }
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