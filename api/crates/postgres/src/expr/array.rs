use sea_query::{Expr, Func, Iden};

pub(crate) struct ArrayExpr;

impl ArrayExpr {
    pub fn length<T1, T2>(arg1: T1, arg2: T2) -> Expr
    where
        T1: Into<Expr>,
        T2: Into<Expr>,
    {
        #[derive(Iden)]
        #[iden = "array_length"]
        struct ArrayLength;

        Func::cust(ArrayLength).arg(arg1).arg(arg2).into()
    }

    pub fn string_to_array<T1, T2>(arg1: T1, arg2: T2) -> Expr
    where
        T1: Into<Expr>,
        T2: Into<Expr>,
    {
        #[derive(Iden)]
        #[iden = "string_to_array"]
        struct StringToArray;

        Func::cust(StringToArray).arg(arg1).arg(arg2).into()
    }

    pub fn unnest<T>(arg: T) -> Expr
    where
        T: Into<Expr>,
    {
        #[derive(Iden)]
        #[iden = "unnest"]
        struct Unnest;

        Func::cust(Unnest).arg(arg).into()
    }
}
