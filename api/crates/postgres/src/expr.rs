use sea_query::{SimpleExpr, Value};

pub(crate) mod aggregate;
pub(crate) mod array;
pub(crate) mod conditional;
pub(crate) mod notify;
pub(crate) mod string;

pub(crate) trait SimpleExprTrait {
    fn to_constant(self) -> SimpleExpr;
}

impl<T> SimpleExprTrait for T
where
    T: Into<Value>,
{
    fn to_constant(self) -> SimpleExpr {
        SimpleExpr::Constant(self.into())
    }
}
