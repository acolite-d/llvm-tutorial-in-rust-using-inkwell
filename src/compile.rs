use std::error::Error;

use inkwell::targets::FileType;

use crate::{
    cli::Cli,
    frontend::{
        lexer::{Lex, Token},
        parser::{parse_definition, parse_extern, parse_top_level_expr},
    },
};
use crate::backend::llvm_backend::{LLVMCodeGen, LLVMContext};


pub fn compile_src<'src>(
    src_code: &'src str, 
    cli: &Cli
) -> Result<(), Box<dyn Error + 'src>> {

    let ctx = inkwell::context::Context::create();
    let llvm_ctx = LLVMContext::new(&ctx, cli.opt_level);

    let mut tokens = src_code.lex().peekable();

    while let Some(token) = tokens.peek() {
        match token {
            Token::Extern => {
                match parse_extern(&mut tokens) {
                    Ok(ast) => { ast.codegen(&llvm_ctx)?; }
                    Err(e) => eprintln!("Error: {}", e),
                }   
            }

            Token::FuncDef => {
                match parse_definition(&mut tokens) {
                    Ok(ast) => { ast.codegen(&llvm_ctx)?; }
                    Err(e) => eprintln!("Error: {}", e),
                }   
            }

            // Eat semicolons and move on
            Token::Semicolon => { tokens.next(); },

            _top_level_expr => {
                match parse_top_level_expr(&mut tokens) {
                    Ok(ast) => { ast.codegen(&llvm_ctx)?; }
                    Err(e) => eprintln!("Error: {}", e),
                }   
            }


        }
    }
    
    // Run the optimization passes on IR in module, output to object/assembly file
    llvm_ctx.run_passes(&cli.passes);

    if cli.asm_p {
        llvm_ctx.compile(&cli.output.as_path(), FileType::Assembly);
    } else {
        llvm_ctx.compile(&cli.output.as_path(), FileType::Object);
    }

    Ok(())
}