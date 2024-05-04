use lexer::{Token, Ops};

pub enum NodeExpr<'input> {
    Expr,
    UnaryExpr(),
    BinaryExpr {
        op: Ops,
        lhs: Box<NodeExpr<'input>>,
        rhs: Box<NodeExpr<'input>>,
    },
    CallExpr {
        callee: &'input str,
        args: Vec<NodeExpr<'input>>
    }
}



#[cfg(test)]
mod test {

    use super::*;

    macro_rules! tree {
        () => {}
    }


    #[test]
    fn expressions() {}

    #[test]
    fn unary_expressions() {}

    #[test]
    fn binary_expressions() {}


}