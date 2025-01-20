use async_trait::async_trait;
use sea_query::{extension::postgres::PgExpr, ColumnDef, Expr, PostgresQueryBuilder, Query, Table};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::{expr::string::StringExpr, external_services::PostgresExternalService};

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
            .add_column(ColumnDef::new(PostgresExternalService::UrlPattern).text())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://bsky\.app/profile/(?<creatorId>[^/?#]+)/post/(?<id>[^/?#]+)(?:[?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("bluesky"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://fantia\.jp/posts/(?<id>\d+)(?:[?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("fantia"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                StringExpr::rtrim(
                    StringExpr::regexp_replace(
                        StringExpr::regexp_replace(
                            Expr::col(PostgresExternalService::BaseUrl),
                            "^https?://",
                            "^https?://",
                        ),
                        r"\.",
                        r"\.",
                    ),
                    "/",
                ).concatenate(r"/@(?<creatorId>[^/?#]+)/(?<id>\d+)(?:[?#].*)?$"),
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("mastodon"))
            .and_where(Expr::col(PostgresExternalService::BaseUrl).is_not_null())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                StringExpr::rtrim(
                    StringExpr::regexp_replace(
                        StringExpr::regexp_replace(
                            Expr::col(PostgresExternalService::BaseUrl),
                            "^https?://",
                            "^https?://",
                        ),
                        r"\.",
                        r"\.",
                    ),
                    "/",
                ).concatenate(r"/notes/(?<id>[^/?#]+)(?:[?#].*)?$"),
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("misskey"))
            .and_where(Expr::col(PostgresExternalService::BaseUrl).is_not_null())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://nijie\.info/view\.php\?id=(?<id>\d+)(?:[?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("nijie"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("pixiv"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://(?:(?<creatorId>[^.]+)\.fanbox\.cc|www\.fanbox\.cc/@(?:[^.]+))/posts/(?<id>\d+)(?:[?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("pixiv_fanbox"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                StringExpr::rtrim(
                    StringExpr::regexp_replace(
                        StringExpr::regexp_replace(
                            Expr::col(PostgresExternalService::BaseUrl),
                            "^https?://",
                            "^https?://",
                        ),
                        r"\.",
                        r"\.",
                    ),
                    "/",
                ).concatenate(r"/notice/(?<id>[^/?#]+)(?:[?#].*)?$"),
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("pleroma"))
            .and_where(Expr::col(PostgresExternalService::BaseUrl).is_not_null())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://seiga\.nicovideo\.jp/seiga/(?<id>\d+)(?:[?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("seiga"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("skeb"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://www\.threads\.net/(?<creatorId>[^/]+)/post/(?<id>[^/$#]+)(?:[?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("threads"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://xfolio\.jp/portfolio/(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("xfolio"))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresExternalService::Table)
            .value(
                PostgresExternalService::UrlPattern,
                r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$",
            )
            .and_where(Expr::col(PostgresExternalService::Kind).eq("x"))
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
