use sea_query::Order;

use domain::repository;

mod expr;

pub mod external_services;
pub mod media;
pub mod replicas;
pub mod sources;
pub mod tag_types;
pub mod tags;

macro_rules! sea_query_uuid_value {
    ($newtype:ty, $innertype:ty) => {
        use ::sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType, ValueTypeErr};
        use ::sqlx::{
            decode::Decode,
            encode::{Encode, IsNull},
            error::BoxDynError,
            postgres::{self, PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef},
            Database, Type,
        };
        use ::uuid::Uuid;

        impl From<$newtype> for Uuid {
            fn from(x: $newtype) -> Self {
                *x.0
            }
        }

        impl From<$newtype> for Value {
            fn from(x: $newtype) -> Self {
                From::<Uuid>::from(x.into())
            }
        }

        impl ValueType for $newtype {
            fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
                <Uuid as ValueType>::try_from(v).map(<$innertype>::from).map(<$newtype>::from)
            }

            fn type_name() -> String {
                <Uuid as ValueType>::type_name()
            }

            fn array_type() -> ArrayType {
                <Uuid as ValueType>::array_type()
            }

            fn column_type() -> ColumnType {
                <Uuid as ValueType>::column_type()
            }
        }

        impl Nullable for $newtype {
            fn null() -> Value {
                <Uuid as Nullable>::null()
            }
        }

        impl Type<postgres::Postgres> for $newtype {
            fn type_info() -> <postgres::Postgres as Database>::TypeInfo {
                <Uuid as Type<postgres::Postgres>>::type_info()
            }
        }

        impl PgHasArrayType for $newtype {
            fn array_type_info() -> PgTypeInfo {
                <Uuid as PgHasArrayType>::array_type_info()
            }
        }

        impl Encode<'_, postgres::Postgres> for $newtype {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
                <Uuid as Encode<'_, postgres::Postgres>>::encode_by_ref(&*self.0, buf)
            }
        }

        impl Decode<'_, postgres::Postgres> for $newtype {
            fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
                <Uuid as Decode<'_, postgres::Postgres>>::decode(value).map(<$innertype>::from).map(<$newtype>::from)
            }
        }
    };
}

pub(crate) use sea_query_uuid_value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrderDirection {
    Ascending,
    Descending,
}

impl From<repository::OrderDirection> for OrderDirection {
    fn from(direction: repository::OrderDirection) -> Self {
        match direction {
            repository::OrderDirection::Ascending => OrderDirection::Ascending,
            repository::OrderDirection::Descending => OrderDirection::Descending,
        }
    }
}

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

    pub(crate) struct DatabaseContext {
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
        let connect_options = PgConnectOptions::new().database(name);
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
