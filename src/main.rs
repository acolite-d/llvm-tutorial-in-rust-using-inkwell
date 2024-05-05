#[macro_use]
extern crate lazy_static;

use std::io::Write;

mod parser;
use parser::ast::interpreter_driver;
use parser::lexer::Lexer;

fn main() {
    interpreter_driver()
}
