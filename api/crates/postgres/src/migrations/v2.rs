use async_trait::async_trait;
use domain::entity::objects::{EntryUrl, EntryUrlPath};
use sea_query::{Expr, LockType, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{migrate::MigrateError, FromRow, Connection, PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::replicas::{PostgresReplica, PostgresReplicaId};

const URL_SCHEME: &str = "file";
const URL_PREFIX: &str = "file://";

pub(super) struct V2Migration;

impl Migration<Postgres> for V2Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "replicas_urlencode_original_url"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres>>> {
        vec_box![ReplicaUrlOperation]
    }
}

struct ReplicaUrlOperation;

#[derive(Debug, FromRow)]
struct PostgresReplicaOriginalUrlRow {
    id: PostgresReplicaId,
    original_url: String,
}

#[async_trait]
impl Operation<Postgres> for ReplicaUrlOperation {
    #[tracing::instrument(skip_all)]
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let mut tx = connection.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresReplica::Id,
                PostgresReplica::OriginalUrl,
            ])
            .from(PostgresReplica::Table)
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let replicas = sqlx::query_as_with::<_, PostgresReplicaOriginalUrlRow, _>(&sql, values)
            .fetch_all(&mut *tx)
            .await?;

        for replica in replicas {
            if let Some(path) = replica.original_url.strip_prefix(URL_PREFIX) {
                let url = EntryUrl::from_path_str(URL_PREFIX, path);
                let (sql, values) = Query::update()
                    .table(PostgresReplica::Table)
                    .value(PostgresReplica::OriginalUrl, url.into_inner())
                    .and_where(Expr::col(PostgresReplica::Id).eq(replica.id))
                    .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let mut tx = connection.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresReplica::Id,
                PostgresReplica::OriginalUrl,
            ])
            .from(PostgresReplica::Table)
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let replicas = sqlx::query_as_with::<_, PostgresReplicaOriginalUrlRow, _>(&sql, values)
            .fetch_all(&mut *tx)
            .await?;

        let mut errors = Vec::new();
        for replica in replicas {
            match EntryUrl::from(replica.original_url).to_path_string(URL_PREFIX) {
                Ok(path) => {
                    let url = EntryUrlPath::from(path).to_url(URL_SCHEME).into_inner();
                    let (sql, values) = Query::update()
                        .table(PostgresReplica::Table)
                        .value(PostgresReplica::OriginalUrl, url)
                        .and_where(Expr::col(PostgresReplica::Id).eq(replica.id))
                        .build_sqlx(PostgresQueryBuilder);

                    sqlx::query_with(&sql, values)
                        .execute(&mut *tx)
                        .await?;
                },
                Err(e) => {
                    tracing::error!("failed to decode URL\nID: {:?}\nError: {e:?}", replica.id);
                    errors.push(e);
                },
            }
        }

        if let Some(e) = errors.pop() {
            tracing::error!("{} error(s) found", errors.len());
            return Err(sqlx::Error::Migrate(Box::new(MigrateError::Source(Box::new(e)))))?;
        }

        tx.commit().await?;
        Ok(())
    }
}
