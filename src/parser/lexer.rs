#[allow(dead_code, unused_variables, unused_imports)]
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError<'src> {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token<'src>),

    #[error("Could not tokenize: {0:?}")]
    CouldNotTokenize(&'src str),
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'src> {
    FuncDef = 1,
    Extern = 2,
    Identifier(&'src str) = 3,
    Number(f64) = 4,
    Operator(Ops) = 5,
    OpenParen = 6,
    ClosedParen = 7,
    Unknown(&'src str) = 8,
    EndOfInput = 0,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ops {
    Plus = 0,
    Minus = 1,
    Mult = 2,
    Div = 3,
    Modulo = 4,
    Assign = 5,
}

pub struct Lexer;

impl Lexer {
    pub fn tokens(input: &str) -> impl Iterator<Item = Token> {
        input
            .split_whitespace()
            .map(Self::tokenize)
            .chain(std::iter::once(Token::EndOfInput))
            .peekable()
    }

    #[inline(always)]
    fn tokenize(string: &str) -> Token {
        use Token::*;

        match string {
            // Keywords
            "def" => FuncDef,
            "extern" => Extern,

            // Operators
            "+" => Operator(Ops::Plus),
            "-" => Operator(Ops::Minus),
            "*" => Operator(Ops::Mult),
            "/" => Operator(Ops::Div),
            "%" => Operator(Ops::Modulo),
            "=" => Operator(Ops::Assign),

            // Parenthesis
            "(" => OpenParen,
            ")" => ClosedParen,

            // Everything else
            text => {
                if let Ok(num) = text.parse::<f64>() {
                    Number(num)
                } else {
                    if text.chars().nth(0).unwrap().is_alphabetic() {
                        Identifier(text)
                    } else {
                        Unknown(text)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Ops::*;
    use Token::*;

    #[test]
    fn lexing_nums() {
        let input = " 2.3  4.654345   700   0.23423  ".to_string();
        let tokens = Lexer::tokens(&input);

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                Number(2.3),
                Number(4.654345),
                Number(700.0),
                Number(0.23423),
                EndOfInput,
            ]
        );
    }

    #[test]
    fn lexing_identifiers() {
        let input = " var1   xyz   GLBAL   some_count ".to_string();
        let tokens = Lexer::tokens(&input);

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                Identifier(&"var1"),
                Identifier(&"xyz"),
                Identifier(&"GLBAL"),
                Identifier(&"some_count"),
                EndOfInput,
            ]
        );
    }

    #[test]
    fn lexing_operators() {
        let input = "   + - * / % =   ".to_string();
        let tokens = Lexer::tokens(&input);

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                Operator(Plus),
                Operator(Minus),
                Operator(Mult),
                Operator(Div),
                Operator(Modulo),
                Operator(Assign),
                EndOfInput,
            ]
        );
    }

    #[test]
    fn lexing_mixed() {
        let input = " def   extern  1.23  x".to_string();
        let tokens = Lexer::tokens(&input);

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![FuncDef, Extern, Number(1.23), Identifier(&"x"), EndOfInput]
        );
    }

    // fn lexing_unknown() {

    // }
}
