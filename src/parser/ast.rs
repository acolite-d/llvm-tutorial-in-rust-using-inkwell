use std::collections::HashMap;
use std::fmt::Debug;
#[allow(unused, dead_code)]
use std::io::Write;
use std::iter::Peekable;
use std::string::ParseError;
use std::{any::Any, hash::Hash};

use crate::parser::lexer::{self, Lex, Ops, ParserError, Token};
use itertools::Itertools;

type ParseResult<'src> = Result<Box<dyn AST>, ParserError<'src>>;

lazy_static! {
    static ref OP_PRECEDENCE: HashMap<Ops, i32> = {
        let mut map = HashMap::new();
        map.insert(Ops::Plus, 20);
        map.insert(Ops::Minus, 20);
        map.insert(Ops::Mult, 40);
        map.insert(Ops::Div, 40);
        map.insert(Ops::Modulo, 40);
        map
    };
}

// trait DynEq {
//     fn as_any(&self) -> &dyn Any;

//     fn dyn_eq(&self, other: &dyn DynEq) -> bool;
// }

// impl<T> DynEq for T
// where
//     T: PartialEq + 'static
// {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn dyn_eq(&self, other: &dyn DynEq) -> bool {
//         other.as_any().downcast_ref::<Self>().map_or(false, |other| other == self)
//     }
// }

// impl PartialEq for Box<dyn AST> {
//     fn eq(&self, other: &Self) -> bool {
//         self.dyn_eq(other)
//     }
// }

pub trait AST: Debug {
    fn codegen(&self) {
        todo!()
    }
}

#[derive(Debug)]
struct NumberExpr(f64);

impl AST for NumberExpr {}

#[derive(Debug)]
struct VariableExpr {
    name: String,
}

impl AST for VariableExpr {}

#[derive(Debug)]
struct BinaryExpr {
    op: Ops,
    left: Box<dyn AST>,
    right: Box<dyn AST>,
}

impl AST for BinaryExpr {}

#[derive(Debug)]
struct CallExpr {
    name: String,
    args: Vec<Box<dyn AST>>,
}

impl AST for CallExpr {}

#[derive(Debug)]
struct Prototype {
    name: String,
    args: Vec<String>,
}

impl AST for Prototype {}

#[derive(Debug)]
struct Function {
    proto: Box<dyn AST>,
    body: Box<dyn AST>,
}

impl AST for Function {}

pub fn build_ast<'src>(
    tokens: impl Iterator<Item = Token<'src>>,
) -> Result<Vec<Box<dyn AST>>, ParserError<'src>> {
    todo!()
}

pub fn interpreter_driver() {
    let mut input_buf = String::new();

    loop {
        print!("Ready >> ");
        std::io::stdout().flush().unwrap();
        let _ = std::io::stdin().read_line(&mut input_buf);

        let mut tokens = input_buf
            .split_whitespace()
            .lex()
            .peekable();

        match tokens.peek() {
            None => continue,

            Some(Token::FuncDef) => match parse_definition(&mut tokens) {
                Ok(ast) => {
                    println!("Parsed a function definition.");
                    dbg!(ast);
                },
                Err(err) => {
                    eprintln!("Error: {}", err);
                    _ = tokens.next();
                }
            },

            Some(Token::Extern) => match parse_extern(&mut tokens) {
                Ok(ast) => {
                    println!("Parsed an extern.");
                    dbg!(ast);
                },
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
                },
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

fn parse_extern<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ParseResult<'src> {
    let _keyword = tokens.next();
    parse_prototype(tokens)
}

fn parse_prototype<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ParseResult<'src> {
    let Some(Token::Identifier(name)) = tokens.next() else {
        return Err(ParserError::ExpectedToken(Token::Identifier(&"")));
    };

    tokens
        .next()
        .filter(|t| matches!(t, Token::OpenParen))
        .ok_or(ParserError::ExpectedToken(Token::OpenParen))?;

    let mut args = vec![];

    while let Some(Token::Identifier(s)) = tokens.peek() {
        args.push(s.to_string());
        let _ = tokens.next();
    }

    let _closed_paren = tokens
        .next()
        .filter(|t| matches!(t, Token::ClosedParen))
        .ok_or(ParserError::ExpectedToken(Token::ClosedParen))?;

    Ok(Box::new(Prototype {
        name: name.to_string(),
        args,
    }))
}

fn parse_definition<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ParseResult<'src> {
    // swallow the def keyword
    let _def = tokens.next();

    // try to parse prototype and body
    let proto = parse_prototype(tokens)?;
    let body = parse_expression(tokens)?;

    Ok(Box::new(Function { proto, body }))
}

fn parse_top_level_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ParseResult<'src> {
    let expr = parse_expression(tokens)?;

    let proto = Box::new(Prototype {
        name: "<anonymous>".to_string(),
        args: vec![],
    });

    Ok(Box::new(Function { proto, body: expr }))
}

fn parse_primary<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ParseResult<'src> {
    match tokens.peek() {
        Some(Token::Identifier(_)) => parse_identifier_expr(tokens),

        Some(Token::Number(_)) => parse_number_expr(tokens),

        Some(Token::OpenParen) => parse_paren_expr(tokens),

        Some(unexpected) => Err(ParserError::UnexpectedToken(*unexpected)),

        None => Err(ParserError::UnexpectedEOI),
    }
}

fn parse_number_expr<'src>(
    tokens: &mut impl Iterator<Item = Token<'src>>,
) -> ParseResult<'src> {
    // tokens.next()
    //     .filter(|t| matches!(t, Token::Number(_)))
    //     .map(|t| Box::new(NumberExpr()))

    if let Some(Token::Number(num)) = tokens.next() {
        Ok(Box::new(NumberExpr(num)))
    } else {
        panic!("Expected next token to be number for parse_number_expr!")
    }
}

fn parse_identifier_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ParseResult<'src> {
    let name = match tokens.next() {
        Some(Token::Identifier(name)) => name,
        _unexpected => panic!("Expected"),
    };

    if let Some(Token::OpenParen) = tokens.peek() {
        let _open_paren = tokens.next();

        loop {
            
        }

        let _closed_paren = tokens.next();

    } else {
        Ok(Box::new(VariableExpr {
            name: name.to_string(),
        }))
    }
}

fn parse_paren_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ParseResult<'src> {
    let _paren = tokens.next();

    let expr = parse_expression(tokens);

    match tokens.next() {
        Some(Token::ClosedParen) => expr,
        Some(unexpected) => Err(ParserError::UnexpectedToken(unexpected)),
        None => Err(ParserError::UnexpectedEOI),
    }
}

fn parse_expression<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ParseResult<'src> {
    let lhs = parse_primary(tokens)?;

    parse_binop_rhs(tokens, lhs, 0)
}

fn get_operator_precedence(token: Token) -> i32 {
    if let Token::Operator(operator) = token {
        OP_PRECEDENCE[&operator]
    } else {
        -1
    }
}

fn parse_binop_rhs<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    mut lhs: Box<dyn AST>,
    expr_prec: i32,
) -> ParseResult<'src> {
    loop {
        let tok_prec = match tokens.peek().copied() {
            Some(token) => get_operator_precedence(token),
            None => return Err(ParserError::UnexpectedEOI),
        };

        if tok_prec < expr_prec {
            return Ok(lhs);
        }

        let Some(next_tok @ Token::Operator(op)) = tokens.next() else {
            panic!("Should be operator here!")
        };

        let mut rhs = parse_primary(tokens)?;

        let next_prec = get_operator_precedence(next_tok);

        if tok_prec < next_prec {
            rhs = parse_binop_rhs(tokens, rhs, tok_prec + 1)?;
        }

        lhs = Box::new(BinaryExpr {
            op,
            left: lhs,
            right: rhs,
        })
    }
}

// fn parse_argument<'src>(token: Token<'src>) -> Result<Box<dyn AST,

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_primaries() {
        // assert_eq!(
        //     ast.unwrap(),
        //     Box::new(VariableExpr{ name: &"someUniqueVar1" })
        // )

        // assert_eq!(
        //     Ok(Box::new(NumberExpr(234.4))),
        //     ast
        // );
    }
}
