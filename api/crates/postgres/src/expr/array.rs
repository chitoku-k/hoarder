use sea_query::{Expr, SimpleExpr};

pub(crate) struct ArrayExpr;

impl ArrayExpr {
    pub fn agg<T>(arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Expr::cust_with_expr("array_agg($1)", arg)
    }

    pub fn unnest<T>(arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Expr::cust_with_expr("unnest($1)", arg)
    }
}
