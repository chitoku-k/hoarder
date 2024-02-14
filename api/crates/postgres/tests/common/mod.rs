use std::{error::Error, future::Future, pin::Pin};

use include_dir::{include_dir, Dir};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use test_context::AsyncTestContext;
use uuid::Uuid;

static FIXTURES: Dir = include_dir!("../database");

type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

pub(crate) struct DatabaseContext {
    pub conn: PgConnection,
    pub pool: PgPool,
    pub name: String,
}

async fn create_database(name: &str) -> Result<PgConnection, BoxDynError> {
    let mut conn = PgConnection::connect_with(&PgConnectOptions::new()).await?;
    conn.execute(&*format!(r#"CREATE DATABASE "{}""#, &name)).await?;

    Ok(conn)
}

async fn connect_database(name: &str) -> Result<PgPool, BoxDynError> {
    let connect_options = PgConnectOptions::new().database(name);
    let pool = PgPoolOptions::new()
        .connect_with(connect_options)
        .await?;

    Ok(pool)
}

async fn setup_database(pool: &PgPool) -> Result<(), BoxDynError> {
    let fixtures = FIXTURES.files()
        .chain(
            FIXTURES
                .get_dir("fixtures")
                .ok_or("fixtures not found")?
                .files()
        );

    for file in fixtures {
        let sql = file.contents_utf8().ok_or("invalid fixture")?;
        pool
            .execute(sql)
            .await
            .map_err(|e| format!("initializing test database failed in {:?}: {e}", file.path()))?;
    }

    Ok(())
}

async fn drop_database(conn: &mut PgConnection, name: &str) -> Result<(), BoxDynError> {
    conn.execute(&*format!(r#"DROP DATABASE "{}" WITH (FORCE)"#, name)).await?;

    Ok(())
}

impl AsyncTestContext for DatabaseContext {
    fn setup<'a>() -> Pin<Box<dyn Future<Output = Self> + Send + 'a>> {
        Box::pin(async move {
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
        })
    }

    fn teardown<'a>(mut self) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.pool.close().await;
            drop_database(&mut self.conn, &self.name).await.unwrap();
        })
    }
}
