use sea_query::{Expr, SimpleExpr};

pub(crate) struct StringExpr;

impl StringExpr {
    pub fn regexp_replace<T1, T2, T3>(arg1: T1, arg2: T2, arg3: T3) -> SimpleExpr
    where
        T1: Into<SimpleExpr>,
        T2: Into<SimpleExpr>,
        T3: Into<SimpleExpr>,
    {
        Expr::cust_with_exprs("regexp_replace($1, $2, $3)", [arg1.into(), arg2.into(), arg3.into()])
    }

    pub fn rtrim<T1, T2>(arg1: T1, arg2: T2) -> SimpleExpr
    where
        T1: Into<SimpleExpr>,
        T2: Into<SimpleExpr>,
    {
        Expr::cust_with_exprs("rtrim($1, $2)", [arg1.into(), arg2.into()])
    }
}
