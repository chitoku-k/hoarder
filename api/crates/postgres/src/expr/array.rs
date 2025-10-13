use sea_query::{Expr, SimpleExpr};

pub(crate) struct ArrayExpr;

impl ArrayExpr {
    pub fn length<T1, T2>(arg1: T1, arg2: T2) -> SimpleExpr
    where
        T1: Into<SimpleExpr>,
        T2: Into<SimpleExpr>,
    {
        Expr::cust_with_exprs("array_length($1, $2)", [arg1.into(), arg2.into()])
    }

    pub fn string_to_array<T1, T2>(arg1: T1, arg2: T2) -> SimpleExpr
    where
        T1: Into<SimpleExpr>,
        T2: Into<SimpleExpr>,
    {
        Expr::cust_with_exprs("string_to_array($1, $2)", [arg1.into(), arg2.into()])
    }

    pub fn unnest<T>(arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Expr::cust_with_expr("unnest($1)", arg)
    }
}
