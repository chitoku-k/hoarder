use sea_query::{Expr, SimpleExpr};

pub struct Distinct;

impl Distinct {
    pub fn arg<T>(arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Expr::cust_with_expr("DISTINCT $1", arg)
    }
}
