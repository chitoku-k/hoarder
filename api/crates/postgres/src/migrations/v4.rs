use async_trait::async_trait;
use sea_query::{BinOper, Expr, PostgresQueryBuilder, Query};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::{migrations::State, sources::PostgresSource};

pub(super) struct V4Migration;

impl Migration<Postgres, State> for V4Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "sources_external_metadata_camelcase"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres, State>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres, State>>> {
        vec_box![SourceExternalMetadataOperation]
    }
}

struct SourceExternalMetadataOperation;

#[async_trait]
impl Operation<Postgres, State> for SourceExternalMetadataOperation {
    async fn up(&self, connection: &mut PgConnection, _state: &State) -> Result<(), Error> {
        const OLD_NAME: &str = "creator_id";
        const OLD_PATH: &str = "{creator_id}";
        const NEW_PATH: &str = "{creatorId}";

        let sql = Query::update()
            .table(PostgresSource::Table)
            .value(
                PostgresSource::ExternalMetadata,
                Expr::cust_with_exprs("jsonb_set($1, $2, $3)", [
                    Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("#-"), OLD_PATH),
                    NEW_PATH.into(),
                    Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("#>"), OLD_PATH),
                ]),
            )
            .and_where(Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("?"), OLD_NAME))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    async fn down(&self, connection: &mut PgConnection, _state: &State) -> Result<(), Error> {
        const OLD_NAME: &str = "creatorId";
        const OLD_PATH: &str = "{creatorId}";
        const NEW_PATH: &str = "{creator_id}";

        let sql = Query::update()
            .table(PostgresSource::Table)
            .value(
                PostgresSource::ExternalMetadata,
                Expr::cust_with_exprs("jsonb_set($1, $2, $3)", [
                    Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("#-"), OLD_PATH),
                    NEW_PATH.into(),
                    Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("#>"), OLD_PATH),
                ]),
            )
            .and_where(Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("?"), OLD_NAME))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}
