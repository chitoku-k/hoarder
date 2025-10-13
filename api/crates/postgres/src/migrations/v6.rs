use async_trait::async_trait;
use sea_query::{extension::postgres::PgExpr, Expr, PostgresQueryBuilder, Query};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::{external_services::PostgresExternalService, sources::PostgresSource};

pub(super) struct V6Migration;

impl Migration<Postgres> for V6Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "external_service_and_sources_twitter_to_x"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres>>> {
        vec_box![
            ExternalServiceTwitterToX,
            SourceTwitterToX,
        ]
    }
}

struct ExternalServiceTwitterToX;

#[async_trait]
impl Operation<Postgres> for ExternalServiceTwitterToX {
    #[tracing::instrument(skip_all)]
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        const OLD_SLUG: &str = "twitter";
        const NEW_SLUG: &str = "x";

        const OLD_KIND: &str = "twitter";
        const NEW_KIND: &str = "x";

        const OLD_NAME: &str = "Twitter";
        const NEW_NAME: &str = "X";

        const OLD_BASE_URL: &str = "https://twitter.com";
        const NEW_BASE_URL: &str = "https://x.com";

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::Slug,
                Expr::cust_with_exprs("replace($1, $2, $3)", [
                    Expr::column(PostgresExternalService::Slug),
                    Expr::value(OLD_SLUG),
                    Expr::value(NEW_SLUG),
                ]),
            )
            .value(PostgresExternalService::Kind, NEW_KIND)
            .value(
                PostgresExternalService::Name,
                Expr::cust_with_exprs("replace($1, $2, $3)", [
                    Expr::column(PostgresExternalService::Name),
                    Expr::value(OLD_NAME),
                    Expr::value(NEW_NAME),
                ]),
            )
            .value(
                PostgresExternalService::BaseUrl,
                Expr::cust_with_exprs("replace($1, $2, $3)", [
                    Expr::column(PostgresExternalService::BaseUrl),
                    Expr::value(OLD_BASE_URL),
                    Expr::value(NEW_BASE_URL),
                ]),
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq(OLD_KIND))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        const OLD_SLUG: &str = "x";
        const NEW_SLUG: &str = "twitter";

        const OLD_KIND: &str = "x";
        const NEW_KIND: &str = "twitter";

        const OLD_NAME: &str = "X";
        const NEW_NAME: &str = "Twitter";

        const OLD_BASE_URL: &str = "https://x.com";
        const NEW_BASE_URL: &str = "https://twitter.com";

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::Slug,
                Expr::cust_with_exprs("replace($1, $2, $3)", [
                    Expr::column(PostgresExternalService::Slug),
                    Expr::value(OLD_SLUG),
                    Expr::value(NEW_SLUG),
                ]),
            )
            .value(PostgresExternalService::Kind, NEW_KIND)
            .value(
                PostgresExternalService::Name,
                Expr::cust_with_exprs("replace($1, $2, $3)", [
                    Expr::column(PostgresExternalService::Name),
                    Expr::value(OLD_NAME),
                    Expr::value(NEW_NAME),
                ]),
            )
            .value(
                PostgresExternalService::BaseUrl,
                Expr::cust_with_exprs("replace($1, $2, $3)", [
                    Expr::column(PostgresExternalService::BaseUrl),
                    Expr::value(OLD_BASE_URL),
                    Expr::value(NEW_BASE_URL),
                ]),
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq(OLD_KIND))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}

struct SourceTwitterToX;

#[async_trait]
impl Operation<Postgres> for SourceTwitterToX {
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        const OLD_TYPE: &str = "twitter";
        const NEW_TYPE: &str = "x";

        let sql = Query::update()
            .table(PostgresSource::Table)
            .value(
                PostgresSource::ExternalMetadata,
                Expr::cust_with_exprs("jsonb_set($1, $2, $3)", [
                    Expr::column(PostgresSource::ExternalMetadata),
                    Expr::value("{type}"),
                    Expr::cust_with_expr("to_jsonb($1::text)", NEW_TYPE),
                ]),
            )
            .value(
                PostgresSource::ExternalMetadataExtra,
                Expr::cust_with_exprs("jsonb_set($1, $2, $3)", [
                    Expr::column(PostgresSource::ExternalMetadataExtra),
                    Expr::value("{type}"),
                    Expr::cust_with_expr("to_jsonb($1::text)", NEW_TYPE),
                ]),
            )
            .and_where(Expr::col(PostgresSource::ExternalMetadata).cast_json_field("type").eq(OLD_TYPE))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        const OLD_TYPE: &str = "x";
        const NEW_TYPE: &str = "twitter";

        let sql = Query::update()
            .table(PostgresSource::Table)
            .value(
                PostgresSource::ExternalMetadata,
                Expr::cust_with_exprs("jsonb_set($1, $2, $3)", [
                    Expr::column(PostgresSource::ExternalMetadata),
                    Expr::value("{type}"),
                    Expr::cust_with_expr("to_jsonb($1::text)", NEW_TYPE),
                ]),
            )
            .value(
                PostgresSource::ExternalMetadataExtra,
                Expr::cust_with_exprs("jsonb_set($1, $2, $3)", [
                    Expr::column(PostgresSource::ExternalMetadataExtra),
                    Expr::value("{type}"),
                    Expr::cust_with_expr("to_jsonb($1::text)", NEW_TYPE),
                ]),
            )
            .and_where(Expr::col(PostgresSource::ExternalMetadata).cast_json_field("type").eq(OLD_TYPE))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}
