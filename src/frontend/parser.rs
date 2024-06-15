use std::collections::HashMap;
use std::iter::Peekable;

use thiserror::Error;

use crate::frontend::{
    ast::*,
    lexer::{Ops, Token},
};

// One of the few global variables I will use here, where the
// tutorial uses many. This is just a hash table of operators
// to their precedence, used in binorph parsing. In the C++
// tutorial, this variable is called "BinopPrecedence"
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

// Few errors here to character what went wrong during the
// parsing process.
#[derive(Error, PartialEq, Debug)]
pub enum ParserError<'src> {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token<'src>),

    #[error("Reached end of input expecting more")]
    UnexpectedEOI,

    #[error("Expected token: {0:?}")]
    ExpectedToken(&'static str),
}

/// external ::= 'extern' prototype
pub fn parse_extern<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> Result<Box<Prototype<'src>>, ParserError<'src>> {
    // Swallow the 'extern' keyword, parse as prototype
    let _keyword = tokens.next();
    parse_prototype(tokens)
}

/// prototype
///   ::= id '(' id* ')'
pub fn parse_prototype<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> Result<Box<Prototype<'src>>, ParserError<'src>> {
    let Some(Token::Identifier(name)) = tokens.next() else {
        panic!("Should only call this function when expecting identifier!")
    };

    tokens
        .next()
        .filter(|t| matches!(t, Token::OpenParen))
        .ok_or(ParserError::ExpectedToken(&"("))?;

    let mut args = vec![];

    while let Some(Token::Identifier(s)) = tokens.peek() {
        args.push(*s);
        let _ = tokens.next();
    }

    let _closed_paren = tokens
        .next()
        .filter(|t| matches!(t, Token::ClosedParen))
        .ok_or(ParserError::ExpectedToken(&")"))?;

    Ok(Box::new(Prototype { name, args }))
}

/// definition ::= 'def' prototype expression
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

/// toplevelexpr ::= expression
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

// Small alias for fallible returns of parsing expressions
type ExprParseResult<'src> = Result<Box<ASTExpr<'src>>, ParserError<'src>>;

/// primary
///   ::= identifierexpr
///   ::= numberexpr
///   ::= parenexpr
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

/// numberexpr ::= number
fn parse_number_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ExprParseResult<'src> {
    if let Some(Token::Number(num)) = tokens.next() {
        Ok(Box::new(ASTExpr::NumberExpr(num)))
    } else {
        panic!("Expected next token to be number for parse_number_expr!")
    }
}

/// identifierexpr
///   ::= identifier
///   ::= identifier '(' expression* ')'
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

/// parenexpr ::= '(' expression ')'
fn parse_paren_expr<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ExprParseResult<'src> {
    // Swallow the open parenthesis
    let _paren = tokens.next();

    // Parse the expression inside it
    let expr = parse_expression(tokens);

    // Should be a closed parenthesis following it.
    match tokens.next() {
        Some(Token::ClosedParen) => expr,
        Some(unexpected) => Err(ParserError::UnexpectedToken(unexpected)),
        None => Err(ParserError::UnexpectedEOI),
    }
}

/// expression
///   ::= primary binoprhs
///
fn parse_expression<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
) -> ExprParseResult<'src> {
    let lhs = parse_primary(tokens)?;

    parse_binop_rhs(tokens, lhs, 0)
}

// Small helper method to fetch the precedence of operator
// from hash table. If the token is not an operator,
// default to -1. Tutorial names this GetTokPrecedence
fn get_token_precedence(token: Token) -> i32 {
    if let Token::Operator(operator) = token {
        OP_PRECEDENCE[&operator]
    } else {
        -1
    }
}

/// binoprhs
///   ::= ('+' primary)*
fn parse_binop_rhs<'src>(
    tokens: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    mut lhs: Box<ASTExpr<'src>>,
    expr_prec: i32,
) -> ExprParseResult<'src> {
    loop {
        let tok_prec = match tokens.peek().copied() {
            Some(token) => get_token_precedence(token),
            None => return Err(ParserError::UnexpectedEOI),
        };

        if tok_prec < expr_prec {
            return Ok(lhs);
        }

        let Some(Token::Operator(op)) = tokens.next() else {
            panic!("FATAL: misuse of of this function in recursive descent!")
        };

        let mut rhs = parse_primary(tokens)?;

        let next_prec = match tokens.peek().copied() {
            Some(token) => get_token_precedence(token),
            None => return Err(ParserError::UnexpectedEOI),
        };

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
    use crate::frontend::lexer::Lex;

    use super::*;
    use ASTExpr::*;
    use Ops::*;

    #[test]
    fn parsing_primary_expressions() {
        let mut tokens = " 23.2 ".lex().peekable();
        let mut res = parse_primary(&mut tokens);

        assert_eq!(res, Ok(Box::new(NumberExpr(23.2))));

        tokens = " myVariable ".lex().peekable();
        res = parse_primary(&mut tokens);

        assert_eq!(res, Ok(Box::new(VariableExpr(&"myVariable"))));

        tokens = " (400.5 - 323.10) ".lex().peekable();
        res = parse_primary(&mut tokens);

        assert_eq!(
            res,
            Ok(Box::new(BinaryExpr {
                op: Minus,
                left: Box::new(NumberExpr(400.5)),
                right: Box::new(NumberExpr(323.10)),
            }))
        );

        tokens = " squareNums(2) ".lex().peekable();
        res = parse_primary(&mut tokens);

        assert_eq!(
            res,
            Ok(Box::new(CallExpr {
                callee: &"squareNums",
                args: vec![Box::new(NumberExpr(2.0))]
            }))
        );

        tokens = " multiParams(6, x, (2 + 2)) ".lex().peekable();
        res = parse_primary(&mut tokens);

        assert_eq!(
            res,
            Ok(Box::new(CallExpr {
                callee: &"multiParams",
                args: vec![
                    Box::new(NumberExpr(6.0)),
                    Box::new(VariableExpr(&"x")),
                    Box::new(BinaryExpr {
                        op: Plus,
                        left: Box::new(NumberExpr(2.0)),
                        right: Box::new(NumberExpr(2.0)),
                    })
                ]
            }))
        );
    }

    #[test]
    fn binary_expression_precedence() {
        // Left takes precedence, precedence here should be
        // (((1+2)-3)+4)
        let mut tokens = " 1 + 2 - 3 + 4;".lex().peekable();
        let mut expr_ast = parse_expression(&mut tokens);

        assert_eq!(
            expr_ast,
            Ok(Box::new(BinaryExpr {
                op: Plus,
                left: Box::new(BinaryExpr {
                    op: Minus,
                    left: Box::new(BinaryExpr {
                        op: Plus,
                        left: Box::new(NumberExpr(1.0)),
                        right: Box::new(NumberExpr(2.0))
                    }),
                    right: Box::new(NumberExpr(3.0)),
                }),
                right: Box::new(NumberExpr(4.0))
            }))
        );

        // The last binary expression " y * z " should take precedence,
        // (x + (y * z))
        tokens = " x + y * z; ".lex().peekable();
        expr_ast = parse_expression(&mut tokens);

        assert_eq!(
            expr_ast,
            Ok(Box::new(BinaryExpr {
                op: Plus,
                left: Box::new(VariableExpr(&"x")),
                right: Box::new(BinaryExpr {
                    op: Mult,
                    left: Box::new(VariableExpr(&"y")),
                    right: Box::new(VariableExpr(&"z")),
                })
            }))
        );

        // But parenthesis can be enforce  explicit binary expression
        // precedence ((x + y) * z)

        tokens = " (x+y)*z;".lex().peekable();
        expr_ast = parse_expression(&mut tokens);

        assert_eq!(
            expr_ast,
            Ok(Box::new(BinaryExpr {
                op: Mult,
                left: Box::new(BinaryExpr {
                    op: Plus,
                    left: Box::new(VariableExpr(&"x")),
                    right: Box::new(VariableExpr(&"y")),
                }),
                right: Box::new(VariableExpr(&"z"))
            }))
        );

        // Here the division expression in middle should take precedence,
        // ((2 + (10 / 5)) - 3)
        tokens = " 2 + 10 / 5 - 3; ".lex().peekable();
        expr_ast = parse_expression(&mut tokens);

        assert_eq!(
            expr_ast,
            Ok(Box::new(BinaryExpr {
                op: Minus,
                left: Box::new(BinaryExpr {
                    op: Plus,
                    left: Box::new(NumberExpr(2.0)),
                    right: Box::new(BinaryExpr {
                        op: Div,
                        left: Box::new(NumberExpr(10.0)),
                        right: Box::new(NumberExpr(5.0)),
                    }),
                }),
                right: Box::new(NumberExpr(3.0)),
            }))
        );
    }

    #[test]
    fn parsing_functions() {
        let mut tokens = "def func1(x y) x * y;".lex().peekable();
        let mut func_ast = parse_definition(&mut tokens);

        assert_eq!(
            func_ast,
            Ok(Box::new(Function {
                proto: Box::new(Prototype {
                    name: &"func1",
                    args: vec![&"x", &"y"]
                }),
                body: Box::new(BinaryExpr {
                    op: Mult,
                    left: Box::new(VariableExpr(&"x")),
                    right: Box::new(VariableExpr(&"y")),
                },)
            }))
        );

        tokens = "def alwaysReturnOne ( ) 1;".lex().peekable();
        func_ast = parse_definition(&mut tokens);

        assert_eq!(
            func_ast,
            Ok(Box::new(Function {
                proto: Box::new(Prototype {
                    name: &"alwaysReturnOne",
                    args: vec![]
                }),
                body: Box::new(NumberExpr(1.0)),
            }))
        );

        tokens = "def func2 (base mid upper) base*mid + upper;"
            .lex()
            .peekable();
        func_ast = parse_definition(&mut tokens);

        assert_eq!(
            func_ast,
            Ok(Box::new(Function {
                proto: Box::new(Prototype {
                    name: &"func2",
                    args: vec![&"base", &"mid", &"upper"]
                }),
                body: Box::new(BinaryExpr {
                    op: Plus,
                    left: Box::new(BinaryExpr {
                        op: Mult,
                        left: Box::new(VariableExpr(&"base")),
                        right: Box::new(VariableExpr(&"mid")),
                    }),
                    right: Box::new(VariableExpr(&"upper")),
                })
            }))
        );
    }
}
