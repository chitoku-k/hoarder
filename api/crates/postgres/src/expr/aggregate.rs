use sea_query::{Expr, Func, Iden};

pub(crate) struct AggregateExpr;

impl AggregateExpr {
    pub fn bool_or<T>(arg: T) -> Expr
    where
        T: Into<Expr>,
    {
        #[derive(Iden)]
        #[iden = "bool_or"]
        struct BoolOr;

        Func::cust(BoolOr).arg(arg).into()
    }
}
