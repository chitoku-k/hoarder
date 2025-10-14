use sea_query::Expr;

pub(crate) struct AggregateExpr;

impl AggregateExpr {
    pub fn bool_or<T>(arg: T) -> Expr
    where
        T: Into<Expr>,
    {
        Expr::cust_with_expr("bool_or($1)", arg)
    }
}
