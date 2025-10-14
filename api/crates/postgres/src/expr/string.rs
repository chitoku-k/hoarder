use sea_query::Expr;

pub(crate) struct StringExpr;

impl StringExpr {
    pub fn regexp_replace<T1, T2, T3>(arg1: T1, arg2: T2, arg3: T3) -> Expr
    where
        T1: Into<Expr>,
        T2: Into<Expr>,
        T3: Into<Expr>,
    {
        Expr::cust_with_exprs("regexp_replace($1, $2, $3)", [arg1.into(), arg2.into(), arg3.into()])
    }

    pub fn rtrim<T1, T2>(arg1: T1, arg2: T2) -> Expr
    where
        T1: Into<Expr>,
        T2: Into<Expr>,
    {
        Expr::cust_with_exprs("rtrim($1, $2)", [arg1.into(), arg2.into()])
    }

    pub fn strpos<T1, T2>(arg1: T1, arg2: T2) -> Expr
    where
        T1: Into<Expr>,
        T2: Into<Expr>,
    {
        Expr::cust_with_exprs("strpos($1, $2)", [arg1.into(), arg2.into()])
    }
}
