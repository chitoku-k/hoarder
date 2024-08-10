use std::error::Error;

use include_dir::{include_dir, Dir};
use postgres::Migrator;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use sqlx_migrator::migrator::{Migrate, Plan};
use test_context::AsyncTestContext;
use uuid::Uuid;

static FIXTURES: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures");

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
    let mut conn = pool.acquire().await?;

    let migrator = Migrator::new().into_boxed_migrator();
    migrator.run(&mut conn, &Plan::apply_all()).await?;

    for file in FIXTURES.files() {
        let sql = file.contents_utf8().ok_or("invalid fixture")?;
        conn
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
