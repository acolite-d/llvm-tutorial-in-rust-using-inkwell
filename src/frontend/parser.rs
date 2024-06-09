use std::collections::HashMap;
use std::iter::Peekable;

use thiserror::Error;

use crate::frontend::{
    ast::*,
    lexer::{Ops, Token, Tokens},
};

lazy_static! {
    static ref OP_PRECEDENCE: HashMap<Ops, i32> = {
        let mut map = HashMap::new();
        map.insert(Ops::Plus, 20);
        map.insert(Ops::Minus, 20);
        map.insert(Ops::Mult, 40);
        map.insert(Ops::Div, 40);
        map
    };
}

#[derive(Error, PartialEq, Debug)]
pub enum ParserError<'src> {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token<'src>),

    #[error("Reached end of input expecting more")]
    UnexpectedEOI,

    #[error("Expected token: {0:?}")]
    ExpectedToken(Token<'src>),
}

pub fn parse_extern<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> Result<Box<Prototype<'src>>, ParserError<'src>> {
    let _keyword = tokens.next();
    parse_prototype(tokens)
}

pub fn parse_prototype<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> Result<Box<Prototype<'src>>, ParserError<'src>> {
    let Some(Token::Identifier(name)) = tokens.next() else {
        panic!("Should only call this function when expecting identifier!")
    };

    tokens
        .next()
        .filter(|t| matches!(t, Token::OpenParen))
        .ok_or(ParserError::ExpectedToken(Token::OpenParen))?;

    let mut args = vec![];

    while let Some(Token::Identifier(s)) = tokens.peek() {
        args.push(*s);
        let _ = tokens.next();
    }

    let _closed_paren = tokens
        .next()
        .filter(|t| matches!(t, Token::ClosedParen))
        .ok_or(ParserError::ExpectedToken(Token::ClosedParen))?;

    Ok(Box::new(Prototype { name, args }))
}

pub fn parse_definition<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> Result<Box<Function<'src>>, ParserError<'src>> {
    // swallow the def keyword
    let _def = tokens.next();

    // try to parse prototype and body
    let proto = parse_prototype(tokens)?;
    let body = parse_expression(tokens)?;

    Ok(Box::new(Function { proto, body }))
}

pub fn parse_top_level_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> Result<Box<Function<'src>>, ParserError<'src>> {
    let expr = parse_expression(tokens)?;

    let proto = Box::new(Prototype {
        name: &"__anonymous_expr",
        args: vec![],
    });

    Ok(Box::new(Function { proto, body: expr }))
}


type ExprParseResult<'src> = Result<Box<ASTExpr<'src>>, ParserError<'src>>;

fn parse_primary<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ExprParseResult<'src> {
    match tokens.peek() {
        Some(Token::Identifier(_)) => parse_identifier_expr(tokens),

        Some(Token::Number(_)) => parse_number_expr(tokens),

        Some(Token::OpenParen) => parse_paren_expr(tokens),

        Some(unexpected) => Err(ParserError::UnexpectedToken(*unexpected)),

        None => Err(ParserError::UnexpectedEOI),
    }
}

fn parse_number_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>
) -> ExprParseResult<'src> {
    if let Some(Token::Number(num)) = tokens.next() {
        Ok(Box::new(ASTExpr::NumberExpr(num)))
    } else {
        panic!("Expected next token to be number for parse_number_expr!")
    }
}

fn parse_identifier_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ExprParseResult<'src> {
    let name = match tokens.next() {
        Some(Token::Identifier(name)) => name,
        _unexpected => panic!("Expected"),
    };

    // Call Expression
    if let Some(Token::OpenParen) = tokens.peek() {
        let _open_paren = tokens.next();

        let mut args = vec![];

        loop {
            if let Some(Token::ClosedParen) = tokens.peek() {
                break;
            }

            parse_expression(tokens).map(|arg_expr| args.push(arg_expr))?;

            if let Some(Token::Comma) = tokens.peek() {
                tokens.next();
                continue;
            }
        }

        let _closed_paren = tokens.next();

        Ok(Box::new(ASTExpr::CallExpr { callee: name, args }))
    } else {
        // Variable Expression
        Ok(Box::new(ASTExpr::VariableExpr(name)))
    }
}

fn parse_paren_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ExprParseResult<'src> {
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
) -> ExprParseResult<'src> {
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
    mut lhs: Box<ASTExpr<'src>>,
    expr_prec: i32,
) -> ExprParseResult<'src> {
    loop {
        let tok_prec = match tokens.peek().copied() {
            Some(token) => get_operator_precedence(token),
            None => return Err(ParserError::UnexpectedEOI),
        };

        if tok_prec < expr_prec {
            return Ok(lhs);
        }

        let Some(next_tok @ Token::Operator(op)) = tokens.next() else {
            panic!("FATAL: misuse of of this function in recursive descent!")
        };

        let mut rhs = parse_primary(tokens)?;

        let next_prec = get_operator_precedence(next_tok);

        if tok_prec < next_prec {
            rhs = parse_binop_rhs(tokens, rhs, tok_prec + 1)?;
        }

        lhs = Box::new(ASTExpr::BinaryExpr {
            op,
            left: lhs,
            right: rhs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Ops::*;
    use Token::*;

    macro_rules! ast_node {
        ( $node:expr ) => {
            Box::new($node) as Box<dyn AST>
        };
    }

    // #[test]
    // fn parsing_primary_expressions() {}

    // #[test]
    // fn parsing_binorphs() {}

    // #[test]
    // fn parsing_functions() {}
}
