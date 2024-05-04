use crate::parser::lexer::{Ops, ParserError, Token};
use std::iter::Peekable;

trait AST {
    fn codegen(&self) {
        todo!()
    }
}

struct NumberExpr(f64);

impl AST for NumberExpr {}

struct VariableExpr<'src> {
    name: &'src str,
}

impl<'src> AST for VariableExpr<'src> {}

struct BinaryExpr {
    op: Ops,
    left: Box<dyn AST>,
    right: Box<dyn AST>,
}

impl AST for BinaryExpr {}

struct CallExpr<'src> {
    name: &'src str,
    args: Vec<Box<dyn AST>>,
}

impl<'src> AST for CallExpr<'src> {}

struct Prototype<'src> {
    name: &'src str,
    args: Vec<&'src str>,
}

impl<'src> AST for Prototype<'src> {}

struct Function<'src> {
    proto: Box<Prototype<'src>>,
    body: Box<dyn AST>,
}

impl<'src> AST for Function<'src> {}

pub fn build_ast<'src>(
    tokens: impl Iterator<Item = Token<'src>>,
) -> Result<Vec<Box<dyn AST>>, ParserError<'src>> {
    todo!()
}

fn parse_primary<'src>(
    tokens: &mut impl Iterator<Item = Token<'src>>,
) -> Result<Box<dyn AST + 'src>, ParserError<'src>> {
    // todo!()
    match tokens.next() {
        Some(Token::Identifier(name)) => parse_identifier_expr(name, tokens),

        Some(Token::Number(num)) => Ok(Box::new(NumberExpr(num))),

        Some(Token::OpenParen) => parse_paren_expr(),

        Some(unexpected) => Err(ParserError::UnexpectedToken(unexpected)),

        None => panic!("EOI reached"),
    }
}

fn parse_identifier_expr<'src>(
    name: &'src str,
    tokens: &mut impl Iterator<Item = Token<'src>>,
) -> Result<Box<dyn AST + 'src>, ParserError<'src>> {
    if let Some(Token::OpenParen) = tokens.next() {
        let args = tokens.take
    } else {
        Ok(Box::new(VariableExpr { name }))
    }
}

fn parse_paren_expr<'src>() -> Result<Box<dyn AST>, ParserError<'src>> {
    todo!()
}
