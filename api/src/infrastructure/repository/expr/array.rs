use sea_query::{Expr, Func, Iden, SimpleExpr};

struct Unnest;

impl Iden for Unnest {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "unnest").unwrap();
    }
}

pub struct ArrayExpr;

impl ArrayExpr {
    pub fn unnest<T>(arg: T) -> Expr
    where
        T: Into<SimpleExpr>,
    {
        Expr::expr(Func::cust(Unnest).arg(arg))
    }
}
