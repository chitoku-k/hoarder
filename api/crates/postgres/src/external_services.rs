use derive_more::{Constructor, From, Into};
use domain::{
    entity::external_services::{ExternalService, ExternalServiceId, ExternalServiceKind},
    error::{Error, ErrorKind, Result},
    repository::{external_services::ExternalServicesRepository, DeleteResult},
};
use futures::TryStreamExt;
use sea_query::{Expr, Iden, LockType, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, PgPool};

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
    kind: String,
    name: String,
    base_url: Option<String>,
    url_pattern: Option<String>,
}

#[derive(Iden)]
pub(crate) enum PostgresExternalService {
    #[iden = "external_services"]
    Table,
    Id,
    Slug,
    Kind,
    Name,
    BaseUrl,
    UrlPattern,
}

sea_query_uuid_value!(PostgresExternalServiceId, ExternalServiceId);

impl From<PostgresExternalServiceRow> for ExternalService {
    fn from(row: PostgresExternalServiceRow) -> Self {
        Self {
            id: row.id.into(),
            slug: row.slug,
            kind: row.kind.into(),
            name: row.name,
            base_url: row.base_url,
            url_pattern: row.url_pattern,
        }
    }
}

impl ExternalServicesRepository for PostgresExternalServicesRepository {
    #[tracing::instrument(skip_all)]
    async fn create(&self, slug: &str, kind: ExternalServiceKind, name: &str, base_url: Option<&str>, url_pattern: Option<&str>) -> Result<ExternalService> {
        let (sql, values) = Query::insert()
            .into_table(PostgresExternalService::Table)
            .columns([
                PostgresExternalService::Slug,
                PostgresExternalService::Kind,
                PostgresExternalService::Name,
                PostgresExternalService::BaseUrl,
                PostgresExternalService::UrlPattern,
            ])
            .values([
                Expr::value(slug),
                Expr::value(kind.to_string()),
                Expr::value(name),
                Expr::value(base_url),
                Expr::value(url_pattern),
            ])
            .map_err(Error::other)?
            .returning(
                Query::returning()
                    .columns([
                        PostgresExternalService::Id,
                        PostgresExternalService::Slug,
                        PostgresExternalService::Kind,
                        PostgresExternalService::Name,
                        PostgresExternalService::BaseUrl,
                        PostgresExternalService::UrlPattern,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        match sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values).fetch_one(&self.pool).await {
            Ok(external_service) => Ok(external_service.into()),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(ErrorKind::ExternalServiceSlugDuplicate { slug: slug.to_string() })?,
            Err(e) => Err(Error::other(e)),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_ids<T>(&self, ids: T) -> Result<Vec<ExternalService>>
    where
        T: Iterator<Item = ExternalServiceId> + Send,
    {
        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Kind,
                PostgresExternalService::Name,
                PostgresExternalService::BaseUrl,
                PostgresExternalService::UrlPattern,
            ])
            .from(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).is_in(ids.map(PostgresExternalServiceId::from)))
            .order_by(PostgresExternalService::Slug, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let external_services = sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        Ok(external_services)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_all(&self) -> Result<Vec<ExternalService>> {
        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Kind,
                PostgresExternalService::Name,
                PostgresExternalService::BaseUrl,
                PostgresExternalService::UrlPattern,
            ])
            .from(PostgresExternalService::Table)
            .order_by(PostgresExternalService::Slug, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let external_services = sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        Ok(external_services)
    }

    #[tracing::instrument(skip_all)]
    async fn update_by_id(&self, id: ExternalServiceId, slug: Option<&str>, name: Option<&str>, base_url: Option<Option<&str>>, url_pattern: Option<Option<&str>>) -> Result<ExternalService> {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Kind,
                PostgresExternalService::Name,
                PostgresExternalService::BaseUrl,
                PostgresExternalService::UrlPattern,
            ])
            .from(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(PostgresExternalServiceId::from(id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let external_service = match sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => ExternalService::from(row),
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::ExternalServiceNotFound { id })?,
            Err(e) => return Err(Error::other(e)),
        };

        let slug = slug.unwrap_or(&external_service.slug);
        let name = name.unwrap_or(&external_service.name);
        let base_url = base_url.unwrap_or(external_service.base_url.as_deref());
        let url_pattern = url_pattern.unwrap_or(external_service.url_pattern.as_deref());

        let (sql, values) = Query::update()
            .table(PostgresExternalService::Table)
            .value(PostgresExternalService::Slug, slug)
            .value(PostgresExternalService::Name, name)
            .value(PostgresExternalService::BaseUrl, base_url)
            .value(PostgresExternalService::UrlPattern, url_pattern)
            .and_where(Expr::col(PostgresExternalService::Id).eq(PostgresExternalServiceId::from(id)))
            .returning(
                Query::returning()
                    .columns([
                        PostgresExternalService::Id,
                        PostgresExternalService::Slug,
                        PostgresExternalService::Kind,
                        PostgresExternalService::Name,
                        PostgresExternalService::BaseUrl,
                        PostgresExternalService::UrlPattern,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let external_service = match sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => ExternalService::from(row),
            Err(e) => return Err(Error::other(e)),
        };

        tx.commit().await.map_err(Error::other)?;
        Ok(external_service)
    }

    #[tracing::instrument(skip_all)]
    async fn delete_by_id(&self, id: ExternalServiceId) -> Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(PostgresExternalServiceId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        let affected = sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(Error::other)?
            .rows_affected();

        match affected {
            0 => Ok(DeleteResult::NotFound),
            count => Ok(DeleteResult::Deleted(count)),
        }
    }
}
