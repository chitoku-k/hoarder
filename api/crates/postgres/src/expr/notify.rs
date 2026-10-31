use sea_query::{Expr, Func, Iden};

pub(crate) struct NotifyExpr;

impl NotifyExpr {
    pub fn notify<T1, T2>(arg1: T1, arg2: T2) -> Expr
    where
        T1: ToString,
        T2: ToString,
    {
        #[derive(Iden)]
        #[iden = "pg_notify"]
        struct PgNotify;

        Func::cust(PgNotify).arg(arg1.to_string()).arg(arg2.to_string()).into()
    }
}
