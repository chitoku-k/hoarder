use sea_query::{Expr, SimpleExpr};

pub struct ArrayExpr;

impl ArrayExpr {
    pub fn unnest<T>(arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Expr::cust_with_expr("unnest($1)", arg)
    }
}
