#![allow(clippy::manual_map)]

use sea_query::Order;

use crate::domain::repository::OrderDirection;

mod expr;

pub mod external_services;
pub mod media;
pub mod replicas;
pub mod sources;
pub mod tag_types;
pub mod tags;

macro_rules! sea_query_uuid_value {
    ($newtype:ty) => {
        impl From<$newtype> for sea_query::Value {
            fn from(x: $newtype) -> Self {
                sea_query::Value::Uuid(Some(Box::new(*x)))
            }
        }
    };
}

pub(crate) use sea_query_uuid_value;

impl From<OrderDirection> for Order {
    fn from(direction: OrderDirection) -> Self {
        match direction {
            OrderDirection::Ascending => Order::Asc,
            OrderDirection::Descending => Order::Desc,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use async_trait::async_trait;
    use include_dir::{include_dir, Dir};
    use sqlx::{
        postgres::{PgConnectOptions, PgPoolOptions},
        Connection, Executor, PgConnection, PgPool,
    };
    use test_context::AsyncTestContext;
    use uuid::Uuid;

    static FIXTURES: Dir = include_dir!("../database");

    pub struct DatabaseContext {
        pub conn: PgConnection,
        pub pool: PgPool,
        pub name: String,
    }

    async fn create_database(name: &str) -> anyhow::Result<PgConnection> {
        let mut conn = PgConnection::connect_with(&PgConnectOptions::new()).await?;
        conn.execute(&*format!(r#"CREATE DATABASE "{}""#, &name)).await?;

        Ok(conn)
    }

    async fn connect_database(name: &str) -> anyhow::Result<PgPool> {
        let connect_options = PgConnectOptions::new().database(&name);
        let pool = PgPoolOptions::new()
            .connect_with(connect_options)
            .await?;

        Ok(pool)
    }

    async fn setup_database(pool: &PgPool) -> anyhow::Result<()> {
        let fixtures = FIXTURES.files()
            .chain(
                FIXTURES
                    .get_dir("fixtures")
                    .context("fixtures not found")?
                    .files()
            );

        for file in fixtures {
            let sql = file.contents_utf8().context("invalid fixture")?;
            pool
                .execute(sql)
                .await
                .context(format!("error initializing test database in {:?}", file.path()))?;
        }

        Ok(())
    }

    async fn drop_database(conn: &mut PgConnection, name: &str) -> anyhow::Result<()> {
        conn.execute(&*format!(r#"DROP DATABASE "{}" WITH (FORCE)"#, name)).await?;

        Ok(())
    }

    #[async_trait]
    impl AsyncTestContext for DatabaseContext {
        async fn setup() -> Self {
            let name = format!("hoarder_{}", Uuid::new_v4());
            let mut conn = create_database(&name).await.unwrap();

            let pool = match connect_database(&name).await {
                Ok(pool) => pool,
                Err(e) => {
                    drop_database(&mut conn, &name).await.unwrap();
                    panic!("{e:?}");
                },
            };

            match setup_database(&pool).await {
                Ok(()) => Self { conn, pool, name },
                Err(e) => {
                    pool.close().await;
                    drop_database(&mut conn, &name).await.unwrap();
                    panic!("{e:?}");
                },
            }
        }

        async fn teardown(mut self) {
            self.pool.close().await;
            drop_database(&mut self.conn, &self.name).await.unwrap();
        }
    }
}
