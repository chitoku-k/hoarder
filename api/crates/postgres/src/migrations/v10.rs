use async_trait::async_trait;
use derive_more::Constructor;
use icu_locale_core::LanguageIdentifier;
use sea_query::{Expr, Index, PostgresQueryBuilder};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::{expr::ExprTrait as _, tags::PostgresTag};

#[derive(Constructor)]
pub(super) struct V10Migration {
    langid: LanguageIdentifier,
}

impl Migration<Postgres> for V10Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "tags_collation"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres>>> {
        vec_box![TagCollationOperation::new(self.langid.clone())]
    }
}

#[derive(Constructor)]
struct TagCollationOperation {
    langid: LanguageIdentifier,
}

#[async_trait]
impl Operation<Postgres> for TagCollationOperation {
    #[tracing::instrument(skip_all)]
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        const COLLATION_NAME: &str = "tags";

        let sql = format!(r#"CREATE COLLATION "{}" (provider = icu, locale = '{}-u-ks-level1', deterministic = false)"#, COLLATION_NAME, self.langid);
        sqlx::query(sqlx::AssertSqlSafe(sql)).execute(&mut *connection).await?;

        let sql = Index::create()
            .name("tags_name_with_collate_idx")
            .if_not_exists()
            .table(PostgresTag::Table)
            .col(Expr::col(PostgresTag::Name).collate(COLLATION_NAME))
            .to_string(PostgresQueryBuilder);

        sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut *connection).await?;

        let sql = Index::create()
            .name("tags_kana_with_collate_idx")
            .if_not_exists()
            .table(PostgresTag::Table)
            .col(Expr::col(PostgresTag::Kana).collate(COLLATION_NAME))
            .to_string(PostgresQueryBuilder);

        sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut *connection).await?;

        let sql = Index::create()
            .name("tags_aliases_with_collate_idx")
            .if_not_exists()
            .table(PostgresTag::Table)
            .col(Expr::col(PostgresTag::Aliases).collate(COLLATION_NAME))
            .to_string(PostgresQueryBuilder);

        sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut *connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        const COLLATION_NAME: &str = "tags";

        let sql = Index::drop()
            .name("tags_name_with_collate_idx")
            .to_string(PostgresQueryBuilder);

        sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut *connection).await?;

        let sql = Index::drop()
            .name("tags_kana_with_collate_idx")
            .to_string(PostgresQueryBuilder);

        sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut *connection).await?;

        let sql = Index::drop()
            .name("tags_aliases_with_collate_idx")
            .to_string(PostgresQueryBuilder);

        sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut *connection).await?;

        let sql = format!(r#"DROP COLLATION "{}""#, COLLATION_NAME);
        sqlx::query(sqlx::AssertSqlSafe(sql)).execute(&mut *connection).await?;

        Ok(())
    }
}
