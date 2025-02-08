use chrono::{DateTime, Utc};
use derive_more::{Constructor, From, Into};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        sources::{Source, SourceId},
    },
    error::{Error, ErrorKind, Result},
    repository::{sources::SourcesRepository, DeleteResult},
};
use futures::{future::ready, TryStreamExt};
use sea_query::{
    extension::postgres::PgExpr,
    Expr, Iden, JoinType, LockType, Order, PostgresQueryBuilder, Query
};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::{types::Json, FromRow, PgPool};

use crate::{
    external_services::{PostgresExternalService, PostgresExternalServiceId, PostgresExternalServiceRow},
    sea_query_uuid_value,
};

#[derive(Clone, Constructor)]
pub struct PostgresSourcesRepository {
    pool: PgPool,
}

#[derive(Clone, Debug, From, Into)]
pub(crate) struct PostgresSourceId(SourceId);

#[derive(Debug, FromRow)]
struct PostgresSourceRow {
    id: PostgresSourceId,
    external_service_id: PostgresExternalServiceId,
    external_metadata: Json<PostgresExternalServiceMetadata>,
    external_metadata_extra: Json<PostgresExternalServiceMetadataExtra>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct PostgresSourceExternalServiceRow {
    source_id: PostgresSourceId,
    source_external_metadata: Json<PostgresExternalServiceMetadata>,
    source_external_metadata_extra: Json<PostgresExternalServiceMetadataExtra>,
    source_created_at: DateTime<Utc>,
    source_updated_at: DateTime<Utc>,
    external_service_id: PostgresExternalServiceId,
    external_service_slug: String,
    external_service_kind: String,
    external_service_name: String,
    external_service_base_url: Option<String>,
    external_service_url_pattern: Option<String>,
}

#[derive(Debug)]
struct PostgresSourceRowAndExternalServiceRow(PostgresSourceRow, PostgresExternalServiceRow);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum PostgresExternalServiceMetadata {
    Bluesky { id: String, creator_id: String },
    Fantia { id: u64 },
    Mastodon { id: u64, creator_id: String },
    Misskey { id: String },
    Nijie { id: u64 },
    Pixiv { id: u64 },
    PixivFanbox { id: u64, creator_id: String },
    Pleroma { id: String },
    Seiga { id: u64 },
    Skeb { id: u64, creator_id: String },
    Threads { id: String },
    Website { url: String },
    X { id: u64 },
    Xfolio { id: u64, creator_id: String },
    #[serde(untagged)]
    Custom(serde_json::Value),
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum PostgresExternalServiceMetadataExtra {
    Bluesky {},
    Fantia {},
    Mastodon {},
    Misskey {},
    Nijie {},
    Pixiv {},
    PixivFanbox {},
    Pleroma {},
    Seiga {},
    Skeb {},
    Threads { creator_id: Option<String> },
    Website {},
    X { creator_id: Option<String> },
    Xfolio {},
    #[serde(untagged)]
    Custom {},
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct PostgresExternalServiceMetadataFull(pub PostgresExternalServiceMetadata, pub PostgresExternalServiceMetadataExtra);

#[derive(Iden)]
pub(crate) enum PostgresSource {
    #[iden = "sources"]
    Table,
    Id,
    ExternalServiceId,
    ExternalMetadata,
    ExternalMetadataExtra,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub(crate) enum PostgresSourceExternalService {
    SourceId,
    SourceExternalMetadata,
    SourceExternalMetadataExtra,
    SourceCreatedAt,
    SourceUpdatedAt,
    ExternalServiceId,
    ExternalServiceSlug,
    ExternalServiceKind,
    ExternalServiceName,
    ExternalServiceBaseUrl,
    ExternalServiceUrlPattern,
}

sea_query_uuid_value!(PostgresSourceId, SourceId);

impl PostgresExternalServiceMetadataFull {
    fn into_inner(self) -> (PostgresExternalServiceMetadata, PostgresExternalServiceMetadataExtra) {
        (self.0, self.1)
    }
}

impl TryFrom<PostgresExternalServiceMetadataFull> for ExternalMetadata {
    type Error = Error;

    fn try_from(metadata: PostgresExternalServiceMetadataFull) -> Result<Self> {
        use PostgresExternalServiceMetadata::*;
        match (metadata.0, metadata.1) {
            (Bluesky { id, creator_id }, PostgresExternalServiceMetadataExtra::Bluesky {}) => Ok(Self::Bluesky { id, creator_id }),
            (Fantia { id }, PostgresExternalServiceMetadataExtra::Fantia {}) => Ok(Self::Fantia { id }),
            (Mastodon { id, creator_id }, PostgresExternalServiceMetadataExtra::Mastodon {}) => Ok(Self::Mastodon { id, creator_id }),
            (Misskey { id }, PostgresExternalServiceMetadataExtra::Misskey {}) => Ok(Self::Misskey { id }),
            (Nijie { id }, PostgresExternalServiceMetadataExtra::Nijie {}) => Ok(Self::Nijie { id }),
            (Pixiv { id }, PostgresExternalServiceMetadataExtra::Pixiv {}) => Ok(Self::Pixiv { id }),
            (PixivFanbox { id, creator_id }, PostgresExternalServiceMetadataExtra::PixivFanbox {}) => Ok(Self::PixivFanbox { id, creator_id }),
            (Pleroma { id }, PostgresExternalServiceMetadataExtra::Pleroma {}) => Ok(Self::Pleroma { id }),
            (Seiga { id }, PostgresExternalServiceMetadataExtra::Seiga {}) => Ok(Self::Seiga { id }),
            (Skeb { id, creator_id }, PostgresExternalServiceMetadataExtra::Skeb {}) => Ok(Self::Skeb { id, creator_id }),
            (Threads { id }, PostgresExternalServiceMetadataExtra::Threads { creator_id }) => Ok(Self::Threads { id, creator_id }),
            (Website { url }, PostgresExternalServiceMetadataExtra::Website {}) => Ok(Self::Website { url }),
            (X { id }, PostgresExternalServiceMetadataExtra::X { creator_id }) => Ok(Self::X { id, creator_id }),
            (Xfolio { id, creator_id }, PostgresExternalServiceMetadataExtra::Xfolio {}) => Ok(Self::Xfolio { id, creator_id }),
            (Custom(v), PostgresExternalServiceMetadataExtra::Custom {}) => Ok(Self::Custom(v.to_string())),
            _ => Err(ErrorKind::SourceMetadataInvalid)?,
        }
    }
}

impl TryFrom<ExternalMetadata> for PostgresExternalServiceMetadataFull {
    type Error = serde_json::Error;

    fn try_from(metadata: ExternalMetadata) -> serde_json::Result<Self> {
        use ExternalMetadata::*;
        match metadata {
            Bluesky { id, creator_id } => Ok(Self(PostgresExternalServiceMetadata::Bluesky { id, creator_id }, PostgresExternalServiceMetadataExtra::Bluesky {})),
            Fantia { id } => Ok(Self(PostgresExternalServiceMetadata::Fantia { id }, PostgresExternalServiceMetadataExtra::Fantia {})),
            Mastodon { id, creator_id } => Ok(Self(PostgresExternalServiceMetadata::Mastodon { id, creator_id }, PostgresExternalServiceMetadataExtra::Mastodon {})),
            Misskey { id } => Ok(Self(PostgresExternalServiceMetadata::Misskey { id }, PostgresExternalServiceMetadataExtra::Misskey {})),
            Nijie { id } => Ok(Self(PostgresExternalServiceMetadata::Nijie { id }, PostgresExternalServiceMetadataExtra::Nijie {})),
            Pixiv { id } => Ok(Self(PostgresExternalServiceMetadata::Pixiv { id }, PostgresExternalServiceMetadataExtra::Pixiv {})),
            PixivFanbox { id, creator_id } => Ok(Self(PostgresExternalServiceMetadata::PixivFanbox { id, creator_id }, PostgresExternalServiceMetadataExtra::PixivFanbox {})),
            Pleroma { id } => Ok(Self(PostgresExternalServiceMetadata::Pleroma { id }, PostgresExternalServiceMetadataExtra::Pleroma {})),
            Seiga { id } => Ok(Self(PostgresExternalServiceMetadata::Seiga { id }, PostgresExternalServiceMetadataExtra::Seiga {})),
            Skeb { id, creator_id } => Ok(Self(PostgresExternalServiceMetadata::Skeb { id, creator_id }, PostgresExternalServiceMetadataExtra::Skeb {})),
            Threads { id, creator_id } => Ok(Self(PostgresExternalServiceMetadata::Threads { id }, PostgresExternalServiceMetadataExtra::Threads { creator_id })),
            Website { url } => Ok(Self(PostgresExternalServiceMetadata::Website { url }, PostgresExternalServiceMetadataExtra::Website {})),
            X { id, creator_id } => Ok(Self(PostgresExternalServiceMetadata::X { id }, PostgresExternalServiceMetadataExtra::X { creator_id })),
            Xfolio { id, creator_id } => Ok(Self(PostgresExternalServiceMetadata::Xfolio { id, creator_id }, PostgresExternalServiceMetadataExtra::Xfolio {})),
            Custom(v) => Ok(Self(PostgresExternalServiceMetadata::Custom(serde_json::from_str(&v)?), PostgresExternalServiceMetadataExtra::Custom {})),
        }
    }
}

impl TryFrom<PostgresSourceRowAndExternalServiceRow> for Source {
    type Error = Error;

    fn try_from(row: PostgresSourceRowAndExternalServiceRow) -> Result<Self> {
        let source_row = row.0;
        let external_service_row = row.1;
        let external_metadata = PostgresExternalServiceMetadataFull(source_row.external_metadata.0, source_row.external_metadata_extra.0);
        let external_metadata = ExternalMetadata::try_from(external_metadata)?;

        Ok(Self {
            id: source_row.id.into(),
            external_service: external_service_row.into(),
            external_metadata,
            created_at: source_row.created_at,
            updated_at: source_row.updated_at,
        })
    }
}

impl TryFrom<PostgresSourceExternalServiceRow> for Source {
    type Error = Error;

    fn try_from(row: PostgresSourceExternalServiceRow) -> Result<Self> {
        let external_metadata = PostgresExternalServiceMetadataFull(row.source_external_metadata.0, row.source_external_metadata_extra.0);
        let external_metadata = ExternalMetadata::try_from(external_metadata)?;

        Ok(Self {
            id: row.source_id.into(),
            external_service: ExternalService {
                id: row.external_service_id.into(),
                slug: row.external_service_slug,
                kind: row.external_service_kind,
                name: row.external_service_name,
                base_url: row.external_service_base_url,
                url_pattern: row.external_service_url_pattern,
            },
            external_metadata,
            created_at: row.source_created_at,
            updated_at: row.source_updated_at,
        })
    }
}

impl SourcesRepository for PostgresSourcesRepository {
    #[tracing::instrument(skip_all)]
    async fn create(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> Result<Source> {
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
            .and_where(Expr::col(PostgresExternalService::Id).eq(PostgresExternalServiceId::from(external_service_id)))
            .build_sqlx(PostgresQueryBuilder);

        let external_service_row = match sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::ExternalServiceNotFound { id: external_service_id })?,
            Err(e) => return Err(Error::other(e)),
        };

        let (external_metadata_value, external_metadata_extra_value) = PostgresExternalServiceMetadataFull::try_from(external_metadata.clone())
            .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?
            .into_inner();

        let external_metadata_value = serde_json::to_value(external_metadata_value)
            .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?;

        let external_metadata_extra_value = serde_json::to_value(external_metadata_extra_value)
            .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?;

        let (sql, values) = Query::insert()
            .into_table(PostgresSource::Table)
            .columns([
                PostgresSource::ExternalServiceId,
                PostgresSource::ExternalMetadata,
                PostgresSource::ExternalMetadataExtra,
            ])
            .values([
                PostgresExternalServiceId::from(external_service_id).into(),
                external_metadata_value.into(),
                external_metadata_extra_value.into(),
            ])
            .map_err(Error::other)?
            .returning(
                Query::returning()
                    .columns([
                        PostgresSource::Id,
                        PostgresSource::ExternalServiceId,
                        PostgresSource::ExternalMetadata,
                        PostgresSource::ExternalMetadataExtra,
                        PostgresSource::CreatedAt,
                        PostgresSource::UpdatedAt
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let row = match sqlx::query_as_with::<_, PostgresSourceRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => row,
            Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::ExternalServiceNotFound { id: external_service_id })?,
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                let id = self.fetch_by_external_metadata(external_service_id, external_metadata).await.unwrap_or_default().map(|s| s.id);
                return Err(ErrorKind::SourceMetadataDuplicate { id })?
            },
            Err(e) => return Err(Error::other(e)),
        };

        let source = Source::try_from(PostgresSourceRowAndExternalServiceRow(row, external_service_row))?;
        source.validate()?;

        tx.commit().await.map_err(Error::other)?;
        Ok(source)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_ids<T>(&self, ids: T) -> Result<Vec<Source>>
    where
        T: Iterator<Item = SourceId> + Send,
    {
        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::Id)), PostgresSourceExternalService::SourceId)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadata)), PostgresSourceExternalService::SourceExternalMetadata)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadataExtra)), PostgresSourceExternalService::SourceExternalMetadataExtra)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::CreatedAt)), PostgresSourceExternalService::SourceCreatedAt)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::UpdatedAt)), PostgresSourceExternalService::SourceUpdatedAt)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Id)), PostgresSourceExternalService::ExternalServiceId)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Slug)), PostgresSourceExternalService::ExternalServiceSlug)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Kind)), PostgresSourceExternalService::ExternalServiceKind)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Name)), PostgresSourceExternalService::ExternalServiceName)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::BaseUrl)), PostgresSourceExternalService::ExternalServiceBaseUrl)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::UrlPattern)), PostgresSourceExternalService::ExternalServiceUrlPattern)
            .from(PostgresSource::Table)
            .join(
                JoinType::InnerJoin,
                PostgresExternalService::Table,
                Expr::col((PostgresExternalService::Table, PostgresExternalService::Id))
                    .equals((PostgresSource::Table, PostgresSource::ExternalServiceId)),
            )
            .and_where(Expr::col((PostgresSource::Table, PostgresSource::Id)).is_in(ids.map(PostgresSourceId::from)))
            .order_by((PostgresExternalService::Table, PostgresExternalService::Slug), Order::Asc)
            .order_by((PostgresSource::Table, PostgresSource::ExternalMetadata), Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let sources = sqlx::query_as_with::<_, PostgresSourceExternalServiceRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_err(Error::other)
            .and_then(|row| ready(row.try_into()))
            .try_collect()
            .await?;

        Ok(sources)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> Result<Option<Source>> {
        let (external_metadata, _) = PostgresExternalServiceMetadataFull::try_from(external_metadata)
            .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?
            .into_inner();

        let external_metadata_value = serde_json::to_value(external_metadata)
            .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?;

        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::Id)), PostgresSourceExternalService::SourceId)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadata)), PostgresSourceExternalService::SourceExternalMetadata)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadataExtra)), PostgresSourceExternalService::SourceExternalMetadataExtra)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::CreatedAt)), PostgresSourceExternalService::SourceCreatedAt)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::UpdatedAt)), PostgresSourceExternalService::SourceUpdatedAt)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Id)), PostgresSourceExternalService::ExternalServiceId)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Slug)), PostgresSourceExternalService::ExternalServiceSlug)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Kind)), PostgresSourceExternalService::ExternalServiceKind)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Name)), PostgresSourceExternalService::ExternalServiceName)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::BaseUrl)), PostgresSourceExternalService::ExternalServiceBaseUrl)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::UrlPattern)), PostgresSourceExternalService::ExternalServiceUrlPattern)
            .from(PostgresSource::Table)
            .join(
                JoinType::InnerJoin,
                PostgresExternalService::Table,
                Expr::col((PostgresExternalService::Table, PostgresExternalService::Id))
                    .equals((PostgresSource::Table, PostgresSource::ExternalServiceId)),
            )
            .and_where(Expr::col(PostgresSource::ExternalServiceId).eq(PostgresExternalServiceId::from(external_service_id)))
            .and_where(Expr::col(PostgresSource::ExternalMetadata).contains(Expr::val(external_metadata_value)))
            .build_sqlx(PostgresQueryBuilder);

        let source = sqlx::query_as_with::<_, PostgresSourceExternalServiceRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::other)?
            .map(TryInto::try_into)
            .transpose()?;

        Ok(source)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_external_metadata_like_id(&self, id: &str) -> Result<Vec<Source>> {
        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::Id)), PostgresSourceExternalService::SourceId)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadata)), PostgresSourceExternalService::SourceExternalMetadata)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadataExtra)), PostgresSourceExternalService::SourceExternalMetadataExtra)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::CreatedAt)), PostgresSourceExternalService::SourceCreatedAt)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::UpdatedAt)), PostgresSourceExternalService::SourceUpdatedAt)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Id)), PostgresSourceExternalService::ExternalServiceId)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Slug)), PostgresSourceExternalService::ExternalServiceSlug)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Kind)), PostgresSourceExternalService::ExternalServiceKind)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Name)), PostgresSourceExternalService::ExternalServiceName)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::BaseUrl)), PostgresSourceExternalService::ExternalServiceBaseUrl)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::UrlPattern)), PostgresSourceExternalService::ExternalServiceUrlPattern)
            .from(PostgresSource::Table)
            .join(
                JoinType::InnerJoin,
                PostgresExternalService::Table,
                Expr::col((PostgresExternalService::Table, PostgresExternalService::Id))
                    .equals((PostgresSource::Table, PostgresSource::ExternalServiceId)),
            )
            .and_where(Expr::col(PostgresSource::ExternalMetadata).cast_json_field("id").eq(id))
            .order_by((PostgresExternalService::Table, PostgresExternalService::Slug), Order::Asc)
            .order_by((PostgresSource::Table, PostgresSource::ExternalMetadata), Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let sources = sqlx::query_as_with::<_, PostgresSourceExternalServiceRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_err(Error::other)
            .and_then(|row| ready(row.try_into()))
            .try_collect()
            .await?;

        Ok(sources)
    }

    #[tracing::instrument(skip_all)]
    async fn update_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> Result<Source> {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresSource::Id,
                PostgresSource::ExternalServiceId,
                PostgresSource::ExternalMetadata,
                PostgresSource::ExternalMetadataExtra,
                PostgresSource::CreatedAt,
                PostgresSource::UpdatedAt,
            ])
            .from(PostgresSource::Table)
            .and_where(Expr::col(PostgresSource::Id).eq(PostgresSourceId::from(id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let row = match sqlx::query_as_with::<_, PostgresSourceRow, _>(&sql, values).fetch_one(&mut *tx).await{
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::SourceNotFound { id })?,
            Err(e) => return Err(Error::other(e)),
        };

        let external_service_id = external_service_id.unwrap_or_else(|| row.external_service_id.into());
        let (external_metadata, external_metadata_extra) = match external_metadata {
            Some(external_metadata) => {
                PostgresExternalServiceMetadataFull::try_from(external_metadata)
                    .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?
                    .into_inner()
            },
            None => {
                PostgresExternalServiceMetadataFull(row.external_metadata.0, row.external_metadata_extra.0).into_inner()
            },
        };
        let (external_metadata_value, external_metadata_extra_value) = (
            serde_json::to_value(external_metadata.clone()).map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?,
            serde_json::to_value(external_metadata_extra.clone()).map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?,
        );

        let (sql, values) = Query::update()
            .table(PostgresSource::Table)
            .value(PostgresSource::ExternalServiceId, Expr::val(PostgresExternalServiceId::from(external_service_id)))
            .value(PostgresSource::ExternalMetadata, Expr::val(external_metadata_value))
            .value(PostgresSource::ExternalMetadataExtra, Expr::val(external_metadata_extra_value))
            .value(PostgresSource::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(PostgresSource::Id).eq(PostgresSourceId::from(id)))
            .returning(
                Query::returning()
                    .columns([
                        PostgresSource::Id,
                        PostgresSource::ExternalServiceId,
                        PostgresSource::ExternalMetadata,
                        PostgresSource::ExternalMetadataExtra,
                        PostgresSource::CreatedAt,
                        PostgresSource::UpdatedAt,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let row = match sqlx::query_as_with::<_, PostgresSourceRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => row,
            Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::ExternalServiceNotFound { id: external_service_id })?,
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                let external_metadata = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(external_metadata, external_metadata_extra))
                    .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?;

                let id = self.fetch_by_external_metadata(external_service_id, external_metadata).await.unwrap_or_default().map(|s| s.id);
                return Err(ErrorKind::SourceMetadataDuplicate { id })?
            },
            Err(e) => return Err(Error::other(e)),
        };

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
            .and_where(Expr::col(PostgresExternalService::Id).eq(row.external_service_id.clone()))
            .build_sqlx(PostgresQueryBuilder);

        let external_service_row = match sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::ExternalServiceNotFound { id: external_service_id })?,
            Err(e) => return Err(Error::other(e)),
        };

        let source = Source::try_from(PostgresSourceRowAndExternalServiceRow(row, external_service_row))?;
        source.validate()?;

        tx.commit().await.map_err(Error::other)?;
        Ok(source)
    }

    #[tracing::instrument(skip_all)]
    async fn delete_by_id(&self, id: SourceId) -> Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresSource::Table)
            .and_where(Expr::col(PostgresSource::Id).eq(PostgresSourceId::from(id)))
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
