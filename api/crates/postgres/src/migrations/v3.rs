use async_trait::async_trait;
use sea_query::{ColumnDef, Expr, Iden, PostgresQueryBuilder, Query, Table};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::external_services::PostgresExternalService;

pub(super) struct V3Migration;

impl Migration<Postgres> for V3Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "external_services_add_kind_base_url"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres>>> {
        vec_box![ExternalServiceKindOperation]
    }
}

struct ExternalServiceKindOperation;

#[derive(Iden)]
enum PostgresExternalServiceTemporary {
    NameOld,
}

#[async_trait]
impl Operation<Postgres> for ExternalServiceKindOperation {
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Table::alter()
            .table(PostgresExternalService::Table)
            .rename_column(PostgresExternalService::Name, PostgresExternalServiceTemporary::NameOld)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresExternalService::Table)
            .add_column(ColumnDef::new(PostgresExternalService::Kind).text())
            .add_column(ColumnDef::new(PostgresExternalService::Name).text())
            .add_column(ColumnDef::new(PostgresExternalService::BaseUrl).text())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(PostgresExternalService::Kind, Expr::col(PostgresExternalService::Slug))
            .value(PostgresExternalService::Name, Expr::col(PostgresExternalServiceTemporary::NameOld))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::BaseUrl,
                Expr::case(Expr::col(PostgresExternalService::Kind).eq("fantia"), "https://fantia.jp")
                    .case(Expr::col(PostgresExternalService::Kind).eq("nijie"), "https://nijie.info")
                    .case(Expr::col(PostgresExternalService::Kind).eq("pixiv"), "https://www.pixiv.net")
                    .case(Expr::col(PostgresExternalService::Kind).eq("seiga"), "https://seiga.nicovideo.jp")
                    .case(Expr::col(PostgresExternalService::Kind).eq("skeb"), "https://skeb.jp")
                    .case(Expr::col(PostgresExternalService::Kind).eq("twitter"), "https://twitter.com"),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresExternalService::Table)
            .drop_column(PostgresExternalServiceTemporary::NameOld)
            .modify_column(ColumnDef::new(PostgresExternalService::Kind).not_null())
            .modify_column(ColumnDef::new(PostgresExternalService::Name).not_null())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Table::alter()
            .table(PostgresExternalService::Table)
            .drop_column(PostgresExternalService::Kind)
            .drop_column(PostgresExternalService::BaseUrl)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}
