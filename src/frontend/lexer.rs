use std::str::SplitWhitespace;

// Our tokens for the Kaleidoscope language, in the original
// tutorial, delimiters like commas, parenthesis, semicolons
// were not in the enum, but where inferred to be understood
// by the lexing process, and stored in token buffer. They
// are included in this enum for transparency.
//
// Unlike the tutorial, we will also use string references
// that will live for source, or the "'src" lifetime.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token<'src> {
    FuncDef = 1,
    Extern = 2,
    Identifier(&'src str) = 3,
    Number(f64) = 4,
    Operator(Ops) = 5,
    OpenParen = 6,
    ClosedParen = 7,
    Comma = 8,
    Semicolon = 9,
    If = 10,
    Then = 11,
    Else = 12,
    For = 13,
    In = 14,
    Unknown(&'src str) = 255,
}

// Operators found here, member field of Token::Operator variant
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ops {
    // General math on floating point values
    Plus = 0,
    Minus = 1,
    Mult = 2,
    Div = 3,

    // Comparison of floating point values
    Eq = 4, // Let's just use "=", which is assignment in most C-based languages, but here it will be comparison
    Neq = 5, // Let's use "!"
    Lt = 6, // "<"
    Gt = 7, // ">"
}

// For strings with no whitespace, need to be able to find out
// if I should lex the entire string, or break it apart into slices
// If the string contains multiple single char tokens, we return true.
impl<'src> Token<'src> {
    fn is_single_char_token(c: char) -> bool {
        match c {
            '+' | '-' | '*' | '/' | ';' | ',' | '(' | ')' | '=' | '!' | '<' | '>' => true,

            _ => false,
        }
    }
}

// Taking any given string slice, and producing a token for it,
// used in Lex trait implementation for str.
#[inline(always)]
fn tokenize(string: &str) -> Token {
    use Token::*;

    assert!(string.len() != 0);

    match string {
        // Keywords
        "def" => FuncDef,
        "extern" => Extern,
        "if" => If,
        "then" => Then,
        "else" => Else,
        "for" => For,
        "in" => In,

        // Operators
        "+" => Operator(Ops::Plus),
        "-" => Operator(Ops::Minus),
        "*" => Operator(Ops::Mult),
        "/" => Operator(Ops::Div),
        "=" => Operator(Ops::Eq),
        "!" => Operator(Ops::Neq),
        "<" => Operator(Ops::Lt),
        ">" => Operator(Ops::Gt),

        // Parenthesis
        "(" => OpenParen,
        ")" => ClosedParen,

        //Delimiters
        "," => Comma,
        ";" => Semicolon,

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

// Our iterator adapter for producing Kaleidoscope tokens,
// the only iterator "I" we really use here is SplitWhitespace, but
// so it is a bit needless to make this generic, but just following
// typical iterator adapter nature.
//
// The iterator I must produce string slices &str, but if it produces
// a slice with multiple tokens in it, we take the first token from it,
// then store the latter part of slice in leftover_slice
#[derive(Debug)]
pub struct Tokens<'src, I> {
    iter: I,
    leftover_slice: Option<&'src str>,
}

impl<'src, I> Iterator for Tokens<'src, I>
where
    I: Iterator<Item = &'src str>,
{
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut slice = self.leftover_slice.take().or_else(|| self.iter.next())?;

        if slice.len() > 1 {
            if let Some(pos) = slice.find(Token::is_single_char_token) {
                if pos != 0 {
                    let (immed, rest) = slice.split_at(pos);
                    slice = immed;
                    self.leftover_slice.replace(rest);
                } else {
                    let (immed, rest) = slice.split_at(1);
                    slice = immed;
                    self.leftover_slice.replace(rest);
                }
            }
        }

        Some(tokenize(slice))
    }
}

// We can apply this trait to produce the iterator for
// Kaleidoscope tokens to foreign type str! Now to lex any
// source code we can.
// let source_code = read_source_code();
// let tokens: Vec<Token> = source_code.lex().collect()
pub trait Lex {
    fn lex(&self) -> Tokens<SplitWhitespace>;
}

impl Lex for str {
    fn lex(&self) -> Tokens<SplitWhitespace> {
        Tokens::new(self.split_whitespace())
    }
}

impl<'src, I> Tokens<'src, I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            leftover_slice: None,
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
        let input = " 2.3  4.654345   700   0.23423  ";
        let tokens = input.lex();

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                Number(2.3),
                Number(4.654345),
                Number(700.0),
                Number(0.23423),
            ]
        );
    }

    #[test]
    fn lexing_identifiers() {
        let input = " var1   xyz   GLBAL   some_count ";
        let tokens = input.lex();

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                Identifier(&"var1"),
                Identifier(&"xyz"),
                Identifier(&"GLBAL"),
                Identifier(&"some_count"),
            ]
        );
    }

    #[test]
    fn lexing_operators() {
        let input = " + - * / ";
        let tokens = input.lex();

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                Operator(Plus),
                Operator(Minus),
                Operator(Mult),
                Operator(Div),
            ]
        );
    }

    #[test]
    fn lexing_calls() {
        let mut input = " func1(2, 5, 10) ";
        let mut tokens = input.lex();

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                Identifier(&"func1"),
                OpenParen,
                Number(2.0),
                Comma,
                Number(5.0),
                Comma,
                Number(10.0),
                ClosedParen,
            ]
        );

        input = " func2 () ";
        tokens = input.lex();

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![Identifier(&"func2"), OpenParen, ClosedParen,]
        );

        input = " func3 (x + 2) ";
        tokens = input.lex();

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                Identifier(&"func3"),
                OpenParen,
                Identifier(&"x"),
                Operator(Ops::Plus),
                Number(2.0),
                ClosedParen,
            ]
        );
    }

    #[test]
    fn lexing_function_defs() {
        let mut input = " def myCalculation(arg1 arg2) ";
        let mut tokens = input.lex();

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![
                FuncDef,
                Identifier(&"myCalculation"),
                OpenParen,
                Identifier(&"arg1"),
                Identifier(&"arg2"),
                ClosedParen,
            ]
        );

        input = " def noParamsCall ( ) ";
        tokens = input.lex();

        assert_eq!(
            tokens.collect::<Vec<Token>>(),
            vec![FuncDef, Identifier(&"noParamsCall"), OpenParen, ClosedParen,]
        );
    }
}
