use anyhow::Context;
use async_trait::async_trait;
use derive_more::Constructor;
use futures::TryStreamExt;
use sea_query::{Expr, Iden, LockType, Order, PostgresQueryBuilder, Query};
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::{
        entity::external_services::{ExternalService, ExternalServiceError, ExternalServiceId},
        repository::{external_services::ExternalServicesRepository, DeleteResult},
    },
    infrastructure::repository::{sea_query_driver_postgres::{bind_query, bind_query_as}, sea_query_uuid_value},
};

#[derive(Clone, Constructor)]
pub struct PostgresExternalServicesRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
pub struct PostgresExternalServiceRow {
    id: Uuid,
    slug: String,
    name: String,
}

#[derive(Iden)]
pub enum PostgresExternalService {
    #[iden = "external_services"]
    Table,
    Id,
    Slug,
    Name,
}

#[derive(Debug, Error)]
pub enum PostgresExternalServiceError {
    #[error("error serializing externalMetadata: {0}")]
    Serialize(serde_json::Error),
    #[error("error deserializing externalMetadata: {0}")]
    Deserialize(serde_json::Error),
}

sea_query_uuid_value!(ExternalServiceId);

impl From<PostgresExternalServiceRow> for ExternalService {
    fn from(row: PostgresExternalServiceRow) -> Self {
        Self {
            id: row.id.into(),
            slug: row.slug,
            name: row.name,
        }
    }
}

#[async_trait]
impl ExternalServicesRepository for PostgresExternalServicesRepository {
    async fn create(&self, slug: &str, name: &str) -> anyhow::Result<ExternalService> {
        let (sql, values) = Query::insert()
            .into_table(PostgresExternalService::Table)
            .columns([
                PostgresExternalService::Slug,
                PostgresExternalService::Name,
            ])
            .values([
                slug.into(),
                name.into(),
            ])?
            .returning(
                Query::returning()
                    .columns([
                        PostgresExternalService::Id,
                        PostgresExternalService::Slug,
                        PostgresExternalService::Name,
                    ])
            )
            .build(PostgresQueryBuilder);

        let external_service = bind_query_as::<PostgresExternalServiceRow>(sqlx::query_as(&sql), &values)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)?;

        Ok(external_service)
    }

    async fn fetch_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<ExternalService>>
    where
        T: IntoIterator<Item = ExternalServiceId> + Send + Sync,
    {
        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Name,
            ])
            .from(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).is_in(ids))
            .order_by(PostgresExternalService::Slug, Order::Asc)
            .build(PostgresQueryBuilder);

        let external_services = bind_query_as::<PostgresExternalServiceRow>(sqlx::query_as(&sql), &values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .try_collect()
            .await?;

        Ok(external_services)
    }

    async fn fetch_all(&self) -> anyhow::Result<Vec<ExternalService>> {
        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Name,
            ])
            .from(PostgresExternalService::Table)
            .order_by(PostgresExternalService::Slug, Order::Asc)
            .build(PostgresQueryBuilder);

        let external_services = bind_query_as::<PostgresExternalServiceRow>(sqlx::query_as(&sql), &values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .try_collect()
            .await?;

        Ok(external_services)
    }

    async fn update_by_id<'a>(&self, id: ExternalServiceId, name: Option<&'a str>) -> anyhow::Result<ExternalService> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Name,
            ])
            .from(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(id))
            .lock(LockType::Update)
            .build(PostgresQueryBuilder);

        let external_service: ExternalService = bind_query_as::<PostgresExternalServiceRow>(sqlx::query_as(&sql), &values)
            .fetch_optional(&mut tx)
            .await?
            .map(Into::into)
            .context(ExternalServiceError::NotFound(id))?;

        let name = name.unwrap_or(&external_service.name);

        let (sql, values) = Query::update()
            .table(PostgresExternalService::Table)
            .value(PostgresExternalService::Name, name.into())
            .and_where(Expr::col(PostgresExternalService::Id).eq(id))
            .returning(
                Query::returning()
                    .columns([
                        PostgresExternalService::Id,
                        PostgresExternalService::Slug,
                        PostgresExternalService::Name,
                    ])
            )
            .build(PostgresQueryBuilder);

        let external_service = bind_query_as::<PostgresExternalServiceRow>(sqlx::query_as(&sql), &values)
            .fetch_one(&mut tx)
            .await
            .map(Into::into)?;

        tx.commit().await?;
        Ok(external_service)
    }

    async fn delete_by_id(&self, id: ExternalServiceId) -> anyhow::Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(id))
            .build(PostgresQueryBuilder);

        let affected = bind_query(sqlx::query(&sql), &values)
            .execute(&self.pool)
            .await?
            .rows_affected();

        match affected {
            0 => Ok(DeleteResult::NotFound),
            count => Ok(DeleteResult::Deleted(count)),
        }
    }
}

#[cfg(test)]
mod tests {
    use compiled_uuid::uuid;
    use pretty_assertions::assert_eq;
    use sqlx::Row;
    use test_context::test_context;

    use crate::infrastructure::repository::tests::DatabaseContext;

    use super::*;

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
        let actual = repository.create("foobar", "FooBar").await.unwrap();

        assert_eq!(actual.slug, "foobar".to_string());
        assert_eq!(actual.name, "FooBar".to_string());

        let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "external_services" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("slug"), "foobar");
        assert_eq!(actual.get::<&str, &str>("name"), "FooBar");
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_fails(ctx: &DatabaseContext) {
        let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
        let actual = repository.create("twitter", "Twitter").await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_ids_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_ids([
            ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        ]).await.unwrap();

        assert_eq!(actual, vec![
            ExternalService {
                id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
            ExternalService {
                id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                slug: "twitter".to_string(),
                name: "Twitter".to_string(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all().await.unwrap();

        assert_eq!(actual, vec![
            ExternalService {
                id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
            ExternalService {
                id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                slug: "skeb".to_string(),
                name: "Skeb".to_string(),
            },
            ExternalService {
                id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                slug: "twitter".to_string(),
                name: "Twitter".to_string(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            None,
        ).await.unwrap();

        assert_eq!(actual.id, ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")));
        assert_eq!(actual.slug, "pixiv".to_string());
        assert_eq!(actual.name, "pixiv".to_string());

        let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "external_services" WHERE "id" = $1"#)
            .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("slug"), "pixiv");
        assert_eq!(actual.get::<&str, &str>("name"), "pixiv");
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_with_name_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            Some("PIXIV"),
        ).await.unwrap();

        assert_eq!(actual.id, ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")));
        assert_eq!(actual.slug, "pixiv".to_string());
        assert_eq!(actual.name, "PIXIV".to_string());

        let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "external_services" WHERE "id" = $1"#)
            .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("slug"), "pixiv");
        assert_eq!(actual.get::<&str, &str>("name"), "PIXIV");
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_fails(ctx: &DatabaseContext) {
        let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            Some("PIXIV"),
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_succeeds(ctx: &DatabaseContext) {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "external_services" WHERE "id" = $1"#)
            .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);

        let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
        let actual = repository.delete_by_id(ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(1));

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "external_services" WHERE "id" = $1"#)
            .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 0);

        let actual = repository.delete_by_id(ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))).await.unwrap();

        assert_eq!(actual, DeleteResult::NotFound);
    }
}
