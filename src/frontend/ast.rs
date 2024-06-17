use crate::frontend::lexer::Ops;

// NOTE TO LEARNERS/DEVELOPERS:
// Previously, the AST followed the tutorial by the letter,
// and created a AST composed of dynamically dispatched expression
// nodes using Rust trait objects. (See previous commit: )
//
// I found it far more efficient and idiomatically Rust,
// to use enum dispatch instead. Dynamic dispatch did
// not seem to be right tool for the job. Reasons why:
// - No user-defined "AST" trait needed, just the enum and structs
// - less indirection with Vtables, likely a whole lot faster
// - Allowed myself to keep associating the 'src lifetime within
//   the AST, where with trait objects, I could not keep it AND implement
//   PartialEq trait for testing w/ comparing trees. No string copies!
//   Space efficient
//
// In the end, the tree is still very much the same, just enum dispatched.
// I use similar names to the C++ classes for the variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ASTExpr<'src> {
    NumberExpr(f64),
    VariableExpr(&'src str),
    BinaryExpr {
        op: Ops,
        left: Box<ASTExpr<'src>>,
        right: Box<ASTExpr<'src>>,
    },
    CallExpr {
        callee: &'src str,
        args: Vec<Box<ASTExpr<'src>>>,
    },
    IfExpr {
        cond: Box<ASTExpr<'src>>,
        then_branch: Box<ASTExpr<'src>>,
        else_branch: Box<ASTExpr<'src>>,
    },
    ForLoopExpr {
        varname: &'src str,
        start: Box<ASTExpr<'src>>,
        end: Box<ASTExpr<'src>>,
        step: Option<Box<ASTExpr<'src>>>,
        body: Box<ASTExpr<'src>>,
    }
}

// Prototype, mimics that off the tutorial C++ class
#[derive(Debug, PartialEq)]
pub struct Prototype<'src> {
    pub name: &'src str,
    pub args: Vec<&'src str>,
}

// Function, mimics that off the tutorial C++ class
#[derive(Debug, PartialEq)]
pub struct Function<'src> {
    pub proto: Box<Prototype<'src>>,
    pub body: Box<ASTExpr<'src>>,
}
