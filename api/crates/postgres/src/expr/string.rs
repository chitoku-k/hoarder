use sea_query::{Expr, Func, Iden};

pub(crate) struct StringExpr;

impl StringExpr {
    pub fn regexp_replace<T1, T2, T3>(arg1: T1, arg2: T2, arg3: T3) -> Expr
    where
        T1: Into<Expr>,
        T2: Into<Expr>,
        T3: Into<Expr>,
    {
        #[derive(Iden)]
        #[iden = "regexp_replace"]
        struct RegexpReplace;

        Func::cust(RegexpReplace).arg(arg1).arg(arg2).arg(arg3).into()
    }

    pub fn rtrim<T1, T2>(arg1: T1, arg2: T2) -> Expr
    where
        T1: Into<Expr>,
        T2: Into<Expr>,
    {
        #[derive(Iden)]
        #[iden = "rtrim"]
        struct Rtrim;

        Func::cust(Rtrim).arg(arg1).arg(arg2).into()
    }

    pub fn strpos<T1, T2>(arg1: T1, arg2: T2) -> Expr
    where
        T1: Into<Expr>,
        T2: Into<Expr>,
    {
        #[derive(Iden)]
        #[iden = "strpos"]
        struct Strpos;

        Func::cust(Strpos).arg(arg1).arg(arg2).into()
    }
}
