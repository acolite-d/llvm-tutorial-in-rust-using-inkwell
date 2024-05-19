use crate::frontend::lexer::Ops;

// Enum dispatch instead? No trait needed
// less indirection with Vtables, faster
// Prevent myself from copying source with
// owned String's and use references, reassociate lifetime.
pub enum ASTExpr<'src> {
    NumberExpr(pub f64),
    VariableExpr(pub &'src str),
    BinaryExpr {
        pub op: Ops,
        pub left: Box<ASTExpr<'src>>,
        pub right: Box<ASTExpr<'src>>,
    },
    CallExpr {
        pub name: &'src str,
        pub args: Vec<Box<ASTExpr<'src>>>,
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
    pub proto: Box<Prototype>,
    pub body: Box<ASTExpr<'src>>,
}