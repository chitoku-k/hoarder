use async_trait::async_trait;
use sea_query::{ColumnDef, Expr, PostgresQueryBuilder, Query, Table};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::external_services::PostgresExternalService;

pub(super) struct V8Migration;

impl Migration<Postgres> for V8Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "external_services_url_pattern"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres>>> {
        vec_box![ExternalServiceUrlPatternOperation]
    }
}


struct ExternalServiceUrlPatternOperation;

#[async_trait]
impl Operation<Postgres> for ExternalServiceUrlPatternOperation {
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Table::alter()
            .table(PostgresExternalService::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Table::alter()
            .table(PostgresExternalService::Table)
            .drop_column(PostgresExternalService::UrlPattern)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}
