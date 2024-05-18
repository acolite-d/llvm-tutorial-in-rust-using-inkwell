use std::fmt::Debug;
use std::any::Any;

use dyn_partial_eq::*;

use crate::backend::llvm_backend::LLVMCodeGen;
use crate::frontend::lexer::Ops;

#[dyn_partial_eq]
pub trait AST: Any + Debug + LLVMCodeGen + 'static {}

#[derive(Debug, DynPartialEq, PartialEq)]
pub struct NumberExpr(pub f64);

impl AST for NumberExpr {}

#[derive(Debug, DynPartialEq, PartialEq)]
pub struct VariableExpr {
    pub name: String,
}

impl AST for VariableExpr {}

#[derive(Debug, DynPartialEq, PartialEq)]
pub struct BinaryExpr {
    pub op: Ops,
    pub left: Box<dyn AST>,
    pub right: Box<dyn AST>,
}

impl AST for BinaryExpr {}

#[derive(Debug, DynPartialEq, PartialEq)]
pub struct CallExpr {
    pub name: String,
    pub args: Vec<Box<dyn AST>>,
}

impl AST for CallExpr {}

#[derive(Debug, DynPartialEq, PartialEq)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
}

impl AST for Prototype {}

#[derive(Debug, DynPartialEq, PartialEq)]
pub struct Function {
    pub proto: Box<Prototype>,
    pub body: Box<dyn AST>,
}

impl AST for Function {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Expr,
    TopLevelExpr,
    Function,
    Extern,
    Program,
}

// pub trait Parse<'src, R>
//     where R: FnMut(Rule) -> ParseResult<'src>
// {
//     fn parse_into_ast(&self, rule: R) -> ParseResult<'src>;
// }
