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
use sea_query::{
    extension::postgres::PgExpr,
    Expr, Iden, JoinType, LockType, PostgresQueryBuilder, Query,
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
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct PostgresSourceExternalServiceRow {
    source_id: PostgresSourceId,
    source_external_metadata: Json<PostgresExternalServiceMetadata>,
    source_created_at: DateTime<Utc>,
    source_updated_at: DateTime<Utc>,
    external_service_id: PostgresExternalServiceId,
    external_service_slug: String,
    external_service_kind: String,
    external_service_name: String,
    external_service_base_url: Option<String>,
}

#[derive(Debug)]
struct PostgresSourceRowAndExternalServiceRow(PostgresSourceRow, PostgresExternalServiceRow);

#[skip_serializing_none]
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
    Threads { id: String, creator_id: Option<String> },
    Twitter { id: u64, creator_id: Option<String> },
    Website { url: String },
    #[serde(untagged)]
    Custom(serde_json::Value),
}

#[derive(Iden)]
pub(crate) enum PostgresSource {
    #[iden = "sources"]
    Table,
    Id,
    ExternalServiceId,
    ExternalMetadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub(crate) enum PostgresSourceExternalService {
    SourceId,
    SourceExternalMetadata,
    SourceCreatedAt,
    SourceUpdatedAt,
    ExternalServiceId,
    ExternalServiceSlug,
    ExternalServiceKind,
    ExternalServiceName,
    ExternalServiceBaseUrl,
}

sea_query_uuid_value!(PostgresSourceId, SourceId);

impl From<PostgresExternalServiceMetadata> for ExternalMetadata {
    fn from(metadata: PostgresExternalServiceMetadata) -> Self {
        use PostgresExternalServiceMetadata::*;
        match metadata {
            Bluesky { id, creator_id } => Self::Bluesky { id, creator_id },
            Fantia { id } => Self::Fantia { id },
            Mastodon { id, creator_id } => Self::Mastodon { id, creator_id },
            Misskey { id } => Self::Misskey { id },
            Nijie { id } => Self::Nijie { id },
            Pixiv { id } => Self::Pixiv { id },
            PixivFanbox { id, creator_id } => Self::PixivFanbox { id, creator_id },
            Pleroma { id } => Self::Pleroma { id },
            Seiga { id } => Self::Seiga { id },
            Skeb { id, creator_id } => Self::Skeb { id, creator_id },
            Threads { id, creator_id } => Self::Threads { id, creator_id },
            Twitter { id, creator_id } => Self::Twitter { id, creator_id },
            Website { url } => Self::Website { url },
            Custom(v) => Self::Custom(v.to_string()),
        }
    }
}

impl TryFrom<ExternalMetadata> for PostgresExternalServiceMetadata {
    type Error = serde_json::Error;

    fn try_from(metadata: ExternalMetadata) -> serde_json::Result<Self> {
        use ExternalMetadata::*;
        match metadata {
            Bluesky { id, creator_id } => Ok(Self::Bluesky { id, creator_id }),
            Fantia { id } => Ok(Self::Fantia { id }),
            Mastodon { id, creator_id } => Ok(Self::Mastodon { id, creator_id }),
            Misskey { id } => Ok(Self::Misskey { id }),
            Nijie { id } => Ok(Self::Nijie { id }),
            Pixiv { id } => Ok(Self::Pixiv { id }),
            PixivFanbox { id, creator_id } => Ok(Self::PixivFanbox { id, creator_id }),
            Pleroma { id } => Ok(Self::Pleroma { id }),
            Seiga { id } => Ok(Self::Seiga { id }),
            Skeb { id, creator_id } => Ok(Self::Skeb { id, creator_id }),
            Threads { id, creator_id } => Ok(Self::Threads { id, creator_id }),
            Twitter { id, creator_id } => Ok(Self::Twitter { id, creator_id }),
            Website { url } => Ok(Self::Website { url }),
            Custom(v) => Ok(Self::Custom(serde_json::from_str(&v)?)),
        }
    }
}

impl From<PostgresSourceRowAndExternalServiceRow> for Source {
    fn from(row: PostgresSourceRowAndExternalServiceRow) -> Self {
        let source_row = row.0;
        let external_service_row = row.1;
        let external_metadata = ExternalMetadata::from(source_row.external_metadata.0);

        Self {
            id: source_row.id.into(),
            external_service: external_service_row.into(),
            external_metadata,
            created_at: source_row.created_at,
            updated_at: source_row.updated_at,
        }
    }
}

impl From<PostgresSourceExternalServiceRow> for Source {
    fn from(row: PostgresSourceExternalServiceRow) -> Self {
        let external_metadata = ExternalMetadata::from(row.source_external_metadata.0);

        Self {
            id: row.source_id.into(),
            external_service: ExternalService {
                id: row.external_service_id.into(),
                slug: row.external_service_slug,
                kind: row.external_service_kind,
                name: row.external_service_name,
                base_url: row.external_service_base_url,
            },
            external_metadata,
            created_at: row.source_created_at,
            updated_at: row.source_updated_at,
        }
    }
}

impl SourcesRepository for PostgresSourcesRepository {
    async fn create(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> Result<Source> {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Kind,
                PostgresExternalService::Name,
                PostgresExternalService::BaseUrl,
            ])
            .from(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(PostgresExternalServiceId::from(external_service_id)))
            .build_sqlx(PostgresQueryBuilder);

        let external_service_row = match sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::ExternalServiceNotFound { id: external_service_id })?,
            Err(e) => return Err(Error::other(e)),
        };

        let external_metadata_value = PostgresExternalServiceMetadata::try_from(external_metadata.clone())
            .and_then(serde_json::to_value)
            .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?;

        let (sql, values) = Query::insert()
            .into_table(PostgresSource::Table)
            .columns([PostgresSource::ExternalServiceId, PostgresSource::ExternalMetadata])
            .values([
                PostgresExternalServiceId::from(external_service_id).into(),
                external_metadata_value.into(),
            ])
            .map_err(Error::other)?
            .returning(
                Query::returning()
                    .columns([
                        PostgresSource::Id,
                        PostgresSource::ExternalServiceId,
                        PostgresSource::ExternalMetadata,
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

        let source = Source::from(PostgresSourceRowAndExternalServiceRow(row, external_service_row));
        source.validate()?;

        tx.commit().await.map_err(Error::other)?;
        Ok(source)
    }

    async fn fetch_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> Result<Option<Source>> {
        let external_metadata_value = PostgresExternalServiceMetadata::try_from(external_metadata)
            .and_then(serde_json::to_value)
            .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?;

        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::Id)), PostgresSourceExternalService::SourceId)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadata)), PostgresSourceExternalService::SourceExternalMetadata)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::CreatedAt)), PostgresSourceExternalService::SourceCreatedAt)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::UpdatedAt)), PostgresSourceExternalService::SourceUpdatedAt)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Id)), PostgresSourceExternalService::ExternalServiceId)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Slug)), PostgresSourceExternalService::ExternalServiceSlug)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Kind)), PostgresSourceExternalService::ExternalServiceKind)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Name)), PostgresSourceExternalService::ExternalServiceName)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::BaseUrl)), PostgresSourceExternalService::ExternalServiceBaseUrl)
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
            .map(Into::into);

        Ok(source)
    }

    async fn update_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> Result<Source> {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresSource::Id,
                PostgresSource::ExternalServiceId,
                PostgresSource::ExternalMetadata,
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
        let external_metadata = external_metadata.unwrap_or_else(|| row.external_metadata.0.into());
        let external_metadata_value = PostgresExternalServiceMetadata::try_from(external_metadata.clone())
            .and_then(serde_json::to_value)
            .map_err(|e| Error::new(ErrorKind::SourceMetadataInvalid, e))?;

        let (sql, values) = Query::update()
            .table(PostgresSource::Table)
            .value(PostgresSource::ExternalServiceId, Expr::val(PostgresExternalServiceId::from(external_service_id)))
            .value(PostgresSource::ExternalMetadata, Expr::val(external_metadata_value))
            .value(PostgresSource::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(PostgresSource::Id).eq(PostgresSourceId::from(id)))
            .returning(
                Query::returning()
                    .columns([
                        PostgresSource::Id,
                        PostgresSource::ExternalServiceId,
                        PostgresSource::ExternalMetadata,
                        PostgresSource::CreatedAt,
                        PostgresSource::UpdatedAt,
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

        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Kind,
                PostgresExternalService::Name,
                PostgresExternalService::BaseUrl,
            ])
            .from(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(row.external_service_id.clone()))
            .build_sqlx(PostgresQueryBuilder);

        let external_service_row = match sqlx::query_as_with::<_, PostgresExternalServiceRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::ExternalServiceNotFound { id: external_service_id })?,
            Err(e) => return Err(Error::other(e)),
        };

        let source = Source::from(PostgresSourceRowAndExternalServiceRow(row, external_service_row));
        source.validate()?;

        tx.commit().await.map_err(Error::other)?;
        Ok(source)
    }

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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn convert_fantia() {
        let metadata = PostgresExternalServiceMetadata::Fantia { id: 123456789 };
        let actual = ExternalMetadata::from(metadata);

        assert_eq!(actual, ExternalMetadata::Fantia { id: 123456789 });

        let metadata = ExternalMetadata::Fantia { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Fantia { id: 123456789 });
    }

    #[test]
    fn convert_nijie() {
        let metadata = PostgresExternalServiceMetadata::Nijie { id: 123456789 };
        let actual = ExternalMetadata::from(metadata);

        assert_eq!(actual, ExternalMetadata::Nijie { id: 123456789 });

        let metadata = ExternalMetadata::Nijie { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Nijie { id: 123456789 });
    }

    #[test]
    fn convert_pixiv() {
        let metadata = PostgresExternalServiceMetadata::Pixiv { id: 123456789 };
        let actual = ExternalMetadata::from(metadata);

        assert_eq!(actual, ExternalMetadata::Pixiv { id: 123456789 });

        let metadata = ExternalMetadata::Pixiv { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Pixiv { id: 123456789 });
    }

    #[test]
    fn convert_pixiv_fanbox() {
        let metadata = PostgresExternalServiceMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() };
        let actual = ExternalMetadata::from(metadata);

        assert_eq!(actual, ExternalMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() });

        let metadata = ExternalMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() });
    }

    #[test]
    fn convert_seiga() {
        let metadata = PostgresExternalServiceMetadata::Seiga { id: 123456789 };
        let actual = ExternalMetadata::from(metadata);

        assert_eq!(actual, ExternalMetadata::Seiga { id: 123456789 });

        let metadata = ExternalMetadata::Seiga { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Seiga { id: 123456789 });
    }

    #[test]
    fn convert_twitter() {
        let metadata = PostgresExternalServiceMetadata::Twitter { id: 123456789, creator_id: Some("creator_01".to_string()) };
        let actual = ExternalMetadata::from(metadata);

        assert_eq!(actual, ExternalMetadata::Twitter { id: 123456789, creator_id: Some("creator_01".to_string()) });

        let metadata = ExternalMetadata::Twitter { id: 123456789, creator_id: Some("creator_01".to_string()) };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Twitter { id: 123456789, creator_id: Some("creator_01".to_string()) });
    }

    #[test]
    fn convert_website() {
        let metadata = PostgresExternalServiceMetadata::Website { url: "https://example.com".to_string() };
        let actual = ExternalMetadata::from(metadata);

        assert_eq!(actual, ExternalMetadata::Website { url: "https://example.com".to_string() });

        let metadata = ExternalMetadata::Website { url: "https://example.com".to_string() };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Website { url: "https://example.com".to_string() });
    }

    #[test]
    fn convert_custom() {
        let metadata = PostgresExternalServiceMetadata::Custom(json!({ "id": 123456789 }));
        let actual = ExternalMetadata::from(metadata);

        assert_eq!(actual, ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string()));

        let metadata = ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string());
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Custom(json!({ "id": 123456789 })));
    }
}
