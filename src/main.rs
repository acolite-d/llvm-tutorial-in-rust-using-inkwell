#[allow(dead_code, unused_variables, unused_imports)]

use std::io::{Read, BufRead};

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
enum Token<'input> {
    EndOfInput              = 0,
    FuncDef                 = 1,
    Extern                  = 2,
    Identifier(&'input str) = 3,
    Number(f64)             = 4,
}

pub struct TokenIter {
    buf: String
}

impl<'input> Iterator for TokenIter {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct Lexer;

impl Lexer {

    // fn read<'input, I: BufRead>(input: I) -> Vec<Token<'input>> {
    //     let mut buf = String::new();
    //     input.read_to_string(&mut buf);

    //     let mut tokens = Vec::new();

    //     for word in buf.split_whitespace() {
    //         let tok = match word {
    //             "def" => Token::FuncDef,

    //             "extern" => Token::Extern,

    //             non_keyword => {
    //                 if let Ok(num) = non_keyword.parse::<f64>() {
    //                     Token::Number(num)
    //                 } else {
    //                     Token::Identifier(non_keyword)
    //                 }
    //             }
    //         };

    //         tokens.push(tok);
    //     }

    //     tokens
    // }

    fn lex_from_string<'input>(string: &'input str) -> Vec<Token<'input>> {
        let mut tokens = Vec::new();

        for word in string.split_whitespace() {
            let tok = match word {
                "def" => Token::FuncDef,

                "extern" => Token::Extern,

                non_keyword => {
                    if let Ok(num) = non_keyword.parse::<f64>() {
                        Token::Number(num)
                    } else {
                        Token::Identifier(non_keyword)
                    }
                }
            };

            tokens.push(tok);
        }

        tokens
    }
}


fn main() {

}


#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn lexing_strings() {
        let mut tokens = Lexer::lex_from_string(" def    extern    1.2   x  ");

        assert_eq!(
            tokens, 
            vec![
                FuncDef, 
                Extern, 
                Number(1.2), 
                Identifier(&"x")
            ]
        );

        tokens = Lexer::lex_from_string(" 2.3  4.654345   700   0.23423  ");

        assert_eq!(
            tokens,
            vec![
                Number(2.3),
                Number(4.654345),
                Number(700.0),
                Number(0.23423),
            ]
        );

        tokens = Lexer::lex_from_string(" var1   xyz   GLBAL   some_count  ");

        assert_eq!(
            tokens,
            vec![
                Identifier(&"var1"),
                Identifier(&"xyz"),
                Identifier(&"GLBAL"),
                Identifier(&"some_count"),
            ]
        );
    }
}