use async_trait::async_trait;
use sea_query::{ColumnDef, Expr, ExprTrait, Iden, PostgresQueryBuilder, Query, Table};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::replicas::{PostgresReplica, PostgresReplicaPhase};

pub(super) struct V9Migration;

impl Migration<Postgres> for V9Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "replicas_phase"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres>>> {
        vec_box![ReplicasPhaseOperation]
    }
}

struct ReplicasPhaseOperation;

#[derive(Iden)]
enum PostgresReplicaTemporary {
    CreatedAtOld,
    UpdatedAtOld,
}

#[async_trait]
impl Operation<Postgres> for ReplicasPhaseOperation {
    #[tracing::instrument(skip_all)]
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Table::alter()
            .table(PostgresReplica::Table)
            .rename_column(PostgresReplica::CreatedAt, PostgresReplicaTemporary::CreatedAtOld)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresReplica::Table)
            .rename_column(PostgresReplica::UpdatedAt, PostgresReplicaTemporary::UpdatedAtOld)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresReplica::Table)
            .add_column(ColumnDef::new(PostgresReplica::Phase).text())
            .add_column(ColumnDef::new(PostgresReplica::CreatedAt).timestamp_with_time_zone())
            .add_column(ColumnDef::new(PostgresReplica::UpdatedAt).timestamp_with_time_zone())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresReplica::Table)
            .value(PostgresReplica::Phase, Expr::val(PostgresReplicaPhase::Ready))
            .value(PostgresReplica::CreatedAt, Expr::col(PostgresReplicaTemporary::CreatedAtOld))
            .value(PostgresReplica::UpdatedAt, Expr::col(PostgresReplicaTemporary::UpdatedAtOld))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresReplica::Table)
            .drop_column(PostgresReplicaTemporary::CreatedAtOld)
            .drop_column(PostgresReplicaTemporary::UpdatedAtOld)
            .modify_column(ColumnDef::new(PostgresReplica::MimeType).text().null())
            .modify_column(ColumnDef::new(PostgresReplica::Width).integer().null())
            .modify_column(ColumnDef::new(PostgresReplica::Height).integer().null())
            .modify_column(ColumnDef::new(PostgresReplica::Phase).text().not_null())
            .modify_column(ColumnDef::new(PostgresReplica::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .modify_column(ColumnDef::new(PostgresReplica::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = r#"
            ALTER TABLE "replicas"
            ADD CONSTRAINT "replicas_phase"
            CHECK (
                ("phase" <> 'ready') OR ("mime_type" IS NOT NULL AND "width" IS NOT NULL AND "height" IS NOT NULL)
            )
        "#;

        sqlx::query(sql).execute(&mut *connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Query::delete()
            .from_table(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Phase).ne(PostgresReplicaPhase::Ready))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = r#"
            ALTER TABLE "replicas"
            DROP CONSTRAINT "replicas_phase"
        "#;

        sqlx::query(sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresReplica::Table)
            .drop_column(PostgresReplica::Phase)
            .modify_column(ColumnDef::new(PostgresReplica::MimeType).text().not_null())
            .modify_column(ColumnDef::new(PostgresReplica::Width).integer().not_null())
            .modify_column(ColumnDef::new(PostgresReplica::Height).integer().not_null())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}
