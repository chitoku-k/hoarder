use sea_query::{Expr, Func, Iden, SimpleExpr};
use serde::Serialize;

struct Unnest;

impl Iden for Unnest {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "unnest").unwrap();
    }
}

pub struct ArrayExpr;

impl ArrayExpr {
    pub fn val<V>(v: V) -> anyhow::Result<SimpleExpr>
    where
        V: Serialize,
    {
        Ok(Expr::cust_with_values(
            "ARRAY(SELECT jsonb_array_elements_text($1::jsonb))",
            [serde_json::to_string(&v)?],
        ))
    }

    pub fn unnest<T>(arg: T) -> Expr
    where
        T: Into<SimpleExpr>,
    {
        Expr::expr(Func::cust(Unnest).arg(arg))
    }
}
