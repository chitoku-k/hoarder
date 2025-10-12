use sea_query::{Expr, SimpleExpr};

pub(crate) struct ConditionalExpr;

impl ConditionalExpr {
    pub fn null_if<T1, T2>(arg1: T1, arg2: T2) -> SimpleExpr
    where
        T1: Into<SimpleExpr>,
        T2: Into<SimpleExpr>,
    {
        Expr::cust_with_exprs("NULLIF($1, $2)", [arg1.into(), arg2.into()])
    }
}
