use anyhow::Context;
use async_trait::async_trait;
use derive_more::{Constructor, From, Into};
use domain::{
    entity::external_services::{ExternalService, ExternalServiceError, ExternalServiceId},
    repository::{external_services::ExternalServicesRepository, DeleteResult},
};
use futures::TryStreamExt;
use sea_query::{Expr, Iden, LockType, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, PgPool};
use thiserror::Error;

use crate::sea_query_uuid_value;

#[derive(Clone, Constructor)]
pub struct PostgresExternalServicesRepository {
    pool: PgPool,
}

#[derive(Clone, Debug, From, Into)]
pub(crate) struct PostgresExternalServiceId(ExternalServiceId);

#[derive(Debug, FromRow)]
pub(crate) struct PostgresExternalServiceRow {
    id: PostgresExternalServiceId,
    slug: String,
    name: String,
}

#[derive(Iden)]
pub(crate) enum PostgresExternalService {
    #[iden = "external_services"]
    Table,
    Id,
    Slug,
    Name,
}

#[derive(Debug, Error)]
pub(crate) enum PostgresExternalServiceError {
    #[error("error serializing externalMetadata: {0}")]
    Serialize(serde_json::Error),
    #[error("error deserializing externalMetadata: {0}")]
    Deserialize(serde_json::Error),
}

sea_query_uuid_value!(PostgresExternalServiceId, ExternalServiceId);

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
            .build_sqlx(PostgresQueryBuilder);

        let external_service = sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?
            .into();

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
            .and_where(Expr::col(PostgresExternalService::Id).is_in(ids.into_iter().map(PostgresExternalServiceId::from)))
            .order_by(PostgresExternalService::Slug, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let external_services = sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values)
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
            .build_sqlx(PostgresQueryBuilder);

        let external_services = sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values)
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
            .and_where(Expr::col(PostgresExternalService::Id).eq(PostgresExternalServiceId::from(id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let external_service: ExternalService = sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values)
            .fetch_optional(&mut *tx)
            .await?
            .map(Into::into)
            .context(ExternalServiceError::NotFound(id))?;

        let name = name.unwrap_or(&external_service.name);

        let (sql, values) = Query::update()
            .table(PostgresExternalService::Table)
            .value(PostgresExternalService::Name, name)
            .and_where(Expr::col(PostgresExternalService::Id).eq(PostgresExternalServiceId::from(id)))
            .returning(
                Query::returning()
                    .columns([
                        PostgresExternalService::Id,
                        PostgresExternalService::Slug,
                        PostgresExternalService::Name,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let external_service = sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values)
            .fetch_one(&mut *tx)
            .await?
            .into();

        tx.commit().await?;
        Ok(external_service)
    }

    async fn delete_by_id(&self, id: ExternalServiceId) -> anyhow::Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(PostgresExternalServiceId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        let affected = sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await?
            .rows_affected();

        match affected {
            0 => Ok(DeleteResult::NotFound),
            count => Ok(DeleteResult::Deleted(count)),
        }
    }
}
