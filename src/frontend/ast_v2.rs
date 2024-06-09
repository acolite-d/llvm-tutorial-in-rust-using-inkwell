use crate::frontend::lexer::Ops;

// Enum dispatch instead? No trait needed
// less indirection with Vtables, faster
// Prevent myself from copying source with
// owned String's and use references, reassociate lifetime.
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
        name: &'src str,
        args: Vec<Box<ASTExpr<'src>>>,
    },
}

// Prototype
#[derive(Debug, PartialEq)]
pub struct Prototype<'src> {
    pub name: &'src str,
    pub args: Vec<&'src str>,
}

// Function
#[derive(Debug, PartialEq)]
pub struct Function<'src> {
    pub proto: Box<Prototype<'src>>,
    pub body: Box<ASTExpr<'src>>,
}