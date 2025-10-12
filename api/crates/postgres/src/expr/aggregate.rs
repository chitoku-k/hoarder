use sea_query::{Expr, SimpleExpr};

pub(crate) struct AggregateExpr;

impl AggregateExpr {
    pub fn bool_or<T>(arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Expr::cust_with_expr("bool_or($1)", arg)
    }
}
