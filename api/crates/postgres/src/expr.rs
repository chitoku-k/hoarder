use sea_query::{Expr, Func, Iden, Value};

pub(crate) mod aggregate;
pub(crate) mod array;
pub(crate) mod conditional;
pub(crate) mod notify;
pub(crate) mod string;

pub(crate) trait ExprTrait {
    fn casefold(self) -> Expr;

    fn collate<TArg>(self, arg: TArg) -> Expr
    where
        TArg: Into<String>;
}

pub(crate) trait ValueTrait {
    fn to_constant(self) -> Expr;
}

impl<T> ExprTrait for T
where
    T: Into<Expr>,
{
    fn casefold(self) -> Expr {
        #[derive(Iden)]
        #[iden = "casefold"]
        struct Casefold;

        Func::cust(Casefold).arg(self).into()
    }

    fn collate<TArg>(self, arg: TArg) -> Expr
    where
        TArg: Into<String>,
    {
        Expr::cust_with_exprs("$1 COLLATE $2", [self.into(), Expr::col(arg.into())])
    }
}

impl<T> ValueTrait for T
where
    T: Into<Value>,
{
    fn to_constant(self) -> Expr {
        Expr::Constant(self.into())
    }
}
