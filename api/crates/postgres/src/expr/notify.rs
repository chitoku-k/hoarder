use sea_query::Expr;

pub(crate) struct NotifyExpr;

impl NotifyExpr {
    pub fn notify<T1, T2>(arg1: T1, arg2: T2) -> Expr
    where
        T1: ToString,
        T2: ToString,
    {
        Expr::cust_with_exprs("pg_notify($1, $2)", [Expr::value(arg1.to_string()), Expr::value(arg2.to_string())])
    }
}
