use std::io::Write;

mod parser;
use parser::lexer::Lexer;

fn main() {
    let mut buffer = String::new();

    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("failed to read input!");

        for token in Lexer::tokens(&buffer) {
            dbg!(token);
        }
    }
}
