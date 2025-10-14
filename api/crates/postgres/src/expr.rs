use sea_query::{Expr, Value};

pub(crate) mod aggregate;
pub(crate) mod array;
pub(crate) mod conditional;
pub(crate) mod notify;
pub(crate) mod string;

pub(crate) trait ExprTrait {
    fn to_constant(self) -> Expr;
}

impl<T> ExprTrait for T
where
    T: Into<Value>,
{
    fn to_constant(self) -> Expr {
        Expr::Constant(self.into())
    }
}
