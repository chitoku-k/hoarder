use sea_query::{Expr, Func, Iden, SimpleExpr};

pub struct Distinct;

impl Distinct {
    pub fn arg<T>(arg: T) -> Expr
    where
        T: Into<SimpleExpr>,
    {
        Expr::expr(Func::cust(Self).arg(arg))
    }
}

impl Iden for Distinct {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "DISTINCT").unwrap();
    }
}
