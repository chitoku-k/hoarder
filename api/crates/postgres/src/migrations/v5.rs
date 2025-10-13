use async_trait::async_trait;
use sea_query::{extension::postgres::PgExpr, BinOper, ColumnDef, Expr, Iden, PostgresQueryBuilder, Query, Table};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::sources::PostgresSource;

const EXTERNAL_SERVICE_EXTRA_FIELD_KINDS: [&str; 2] = ["threads", "twitter"];

pub(super) struct V5Migration;

impl Migration<Postgres> for V5Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "sources_external_metadata_extra"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres>>> {
        vec_box![SourceExternalMetadataExtraOperation]
    }
}

struct SourceExternalMetadataExtraOperation;

#[derive(Iden)]
enum PostgresSourceTemporary {
    CreatedAtOld,
    UpdatedAtOld,
}

#[async_trait]
impl Operation<Postgres> for SourceExternalMetadataExtraOperation {
    #[tracing::instrument(skip_all)]
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Table::alter()
            .table(PostgresSource::Table)
            .rename_column(PostgresSource::CreatedAt, PostgresSourceTemporary::CreatedAtOld)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresSource::Table)
            .rename_column(PostgresSource::UpdatedAt, PostgresSourceTemporary::UpdatedAtOld)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresSource::Table)
            .add_column(ColumnDef::new(PostgresSource::ExternalMetadataExtra).json_binary())
            .add_column(ColumnDef::new(PostgresSource::CreatedAt).timestamp_with_time_zone())
            .add_column(ColumnDef::new(PostgresSource::UpdatedAt).timestamp_with_time_zone())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresSource::Table)
            .value(
                PostgresSource::ExternalMetadataExtra,
                Expr::case(
                    Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("?"), "type"),
                    Expr::cust_with_exprs("jsonb_build_object($1, $2)", [
                        Expr::value("type"),
                        Expr::col(PostgresSource::ExternalMetadata).cast_json_field("type"),
                    ]),
                ).finally("{}"),
            )
            .value(PostgresSource::CreatedAt, Expr::col(PostgresSourceTemporary::CreatedAtOld))
            .value(PostgresSource::UpdatedAt, Expr::col(PostgresSourceTemporary::UpdatedAtOld))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresSource::Table)
            .value(
                PostgresSource::ExternalMetadataExtra,
                Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("#-"), "{id}"),
            )
            .value(
                PostgresSource::ExternalMetadata,
                Expr::col(PostgresSource::ExternalMetadata).binary(BinOper::Custom("#-"), "{creatorId}"),
            )
            .and_where(
                Expr::expr(Expr::col(PostgresSource::ExternalMetadata).cast_json_field("type"))
                    .is_in(EXTERNAL_SERVICE_EXTRA_FIELD_KINDS),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresSource::Table)
            .drop_column(PostgresSourceTemporary::CreatedAtOld)
            .drop_column(PostgresSourceTemporary::UpdatedAtOld)
            .modify_column(ColumnDef::new(PostgresSource::ExternalMetadataExtra).json_binary().not_null())
            .modify_column(ColumnDef::new(PostgresSource::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .modify_column(ColumnDef::new(PostgresSource::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Query::update()
            .table(PostgresSource::Table)
            .value(
                PostgresSource::ExternalMetadata,
                Expr::col(PostgresSource::ExternalMetadata).concatenate(Expr::col(PostgresSource::ExternalMetadataExtra)),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresSource::Table)
            .drop_column(PostgresSource::ExternalMetadataExtra)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}
