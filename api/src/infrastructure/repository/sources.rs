use anyhow::Context;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use derive_more::Constructor;
use futures::TryStreamExt;
use sea_query::{Expr, Func, Iden, JoinType, LockType, PostgresQueryBuilder, Query};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, PgPool};
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
            sources::{Source, SourceError, SourceId},
        },
        repository::{sources::SourcesRepository, DeleteResult},
    },
    infrastructure::repository::{
        external_services::{PostgresExternalService, PostgresExternalServiceError, PostgresExternalServiceRow},
        sea_query_driver_postgres::{bind_query, bind_query_as}, sea_query_uuid_value,
    },
};

#[derive(Clone, Constructor)]
pub struct PostgresSourcesRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
struct PostgresSourceRow {
    id: Uuid,
    external_service_id: Uuid,
    external_metadata: Json<PostgresExternalServiceMetadata>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, FromRow)]
struct PostgresSourceExternalServiceRow {
    source_id: Uuid,
    source_external_metadata: Json<PostgresExternalServiceMetadata>,
    source_created_at: NaiveDateTime,
    source_updated_at: NaiveDateTime,
    external_service_id: Uuid,
    external_service_slug: String,
    external_service_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PostgresExternalServiceMetadata {
    Fantia { id: u64 },
    Nijie { id: u64 },
    Pixiv { id: u64 },
    PixivFanbox { id: u64, creator_id: String },
    Seiga { id: u64 },
    Skeb { id: u64, creator_id: String },
    Twitter { id: u64 },
    Website { url: String },
    Custom(serde_json::Value),
}

#[derive(Iden)]
pub enum PostgresSource {
    #[iden = "sources"]
    Table,
    Id,
    ExternalServiceId,
    ExternalMetadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum PostgresSourceExternalService {
    SourceId,
    SourceExternalMetadata,
    SourceCreatedAt,
    SourceUpdatedAt,
    ExternalServiceId,
    ExternalServiceSlug,
    ExternalServiceName,
}

sea_query_uuid_value!(SourceId);

impl TryFrom<PostgresExternalServiceMetadata> for ExternalMetadata {
    type Error = serde_json::Error;

    fn try_from(metadata: PostgresExternalServiceMetadata) -> serde_json::Result<Self> {
        use PostgresExternalServiceMetadata::*;
        match metadata {
            Fantia { id } => Ok(Self::Fantia { id }),
            Nijie { id } => Ok(Self::Nijie { id }),
            Pixiv { id } => Ok(Self::Pixiv { id }),
            PixivFanbox { id, creator_id } => Ok(Self::PixivFanbox { id, creator_id }),
            Seiga { id } => Ok(Self::Seiga { id }),
            Skeb { id, creator_id } => Ok(Self::Skeb { id, creator_id }),
            Twitter { id } => Ok(Self::Twitter { id }),
            Website { url } => Ok(Self::Website { url }),
            Custom(v) => Ok(Self::Custom(serde_json::to_string(&v)?)),
        }
    }
}

impl TryFrom<ExternalMetadata> for PostgresExternalServiceMetadata {
    type Error = serde_json::Error;

    fn try_from(metadata: ExternalMetadata) -> serde_json::Result<Self> {
        use ExternalMetadata::*;
        match metadata {
            Fantia { id } => Ok(Self::Fantia { id }),
            Nijie { id } => Ok(Self::Nijie { id }),
            Pixiv { id } => Ok(Self::Pixiv { id }),
            PixivFanbox { id, creator_id } => Ok(Self::PixivFanbox { id, creator_id }),
            Seiga { id } => Ok(Self::Seiga { id }),
            Skeb { id, creator_id } => Ok(Self::Skeb { id, creator_id }),
            Twitter { id } => Ok(Self::Twitter { id }),
            Website { url } => Ok(Self::Website { url }),
            Custom(v) => Ok(Self::Custom(serde_json::from_str(&v)?)),
        }
    }
}

impl TryFrom<(PostgresSourceRow, ExternalService)> for Source {
    type Error = serde_json::Error;

    fn try_from((row, external_service): (PostgresSourceRow, ExternalService)) -> serde_json::Result<Self> {
        let external_metadata = ExternalMetadata::try_from(row.external_metadata.0)?;

        Ok(Self {
            id: row.id.into(),
            external_service,
            external_metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<PostgresSourceExternalServiceRow> for Source {
    type Error = serde_json::Error;

    fn try_from(row: PostgresSourceExternalServiceRow) -> serde_json::Result<Self> {
        let external_metadata = ExternalMetadata::try_from(row.source_external_metadata.0)?;

        Ok(Self {
            id: row.source_id.into(),
            external_service: ExternalService {
                id: row.external_service_id.into(),
                slug: row.external_service_slug,
                name: row.external_service_name,
            },
            external_metadata,
            created_at: row.source_created_at,
            updated_at: row.source_updated_at,
        })
    }
}

#[async_trait]
impl SourcesRepository for PostgresSourcesRepository {
    async fn create(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Source> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Name,
            ])
            .from(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(external_service_id))
            .build(PostgresQueryBuilder);

        let external_service: ExternalService = bind_query_as::<PostgresExternalServiceRow>(sqlx::query_as(&sql), &values)
            .fetch_one(&mut tx)
            .await?
            .into();

        let external_metadata = PostgresExternalServiceMetadata::try_from(external_metadata)?;
        let external_metadata = match serde_json::to_value(&external_metadata) {
            Ok(value) => Some(value),
            Err(e) => return Err(PostgresExternalServiceError::Serialize(e))?,
        };

        let (sql, values) = Query::insert()
            .into_table(PostgresSource::Table)
            .columns([PostgresSource::ExternalServiceId, PostgresSource::ExternalMetadata])
            .values([external_service_id.into(), external_metadata.into()])?
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
            .build(PostgresQueryBuilder);

        let row = bind_query_as::<PostgresSourceRow>(sqlx::query_as(&sql), &values)
            .fetch_one(&mut tx)
            .await?;

        let source = match Source::try_from((row, external_service)) {
            Ok(source) => {
                source.validate()?;
                source
            },
            Err(e) => return Err(PostgresExternalServiceError::Deserialize(e))?,
        };

        tx.commit().await?;
        Ok(source)
    }

    async fn fetch_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Vec<Source>> {
        let external_metadata = PostgresExternalServiceMetadata::try_from(external_metadata)?;
        let external_metadata = match serde_json::to_value(&external_metadata) {
            Ok(value) => value,
            Err(e) => return Err(PostgresExternalServiceError::Serialize(e))?,
        };

        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::Id)), PostgresSourceExternalService::SourceId)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadata)), PostgresSourceExternalService::SourceExternalMetadata)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::CreatedAt)), PostgresSourceExternalService::SourceCreatedAt)
            .expr_as(Expr::col((PostgresSource::Table, PostgresSource::UpdatedAt)), PostgresSourceExternalService::SourceUpdatedAt)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Id)), PostgresSourceExternalService::ExternalServiceId)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Slug)), PostgresSourceExternalService::ExternalServiceSlug)
            .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Name)), PostgresSourceExternalService::ExternalServiceName)
            .from(PostgresSource::Table)
            .join(
                JoinType::InnerJoin,
                PostgresExternalService::Table,
                Expr::col((PostgresExternalService::Table, PostgresExternalService::Id))
                    .equals(PostgresSource::Table, PostgresSource::ExternalServiceId)
            )
            .and_where(Expr::col(PostgresSource::ExternalServiceId).eq(external_service_id))
            .and_where(Expr::col(PostgresSource::ExternalMetadata).contains(Expr::val(external_metadata)))
            .build(PostgresQueryBuilder);

        let mut sources = Vec::new();
        let mut stream = bind_query_as::<PostgresSourceExternalServiceRow>(sqlx::query_as(&sql), &values).fetch(&self.pool);

        while let Some(row) = stream.try_next().await? {
            let source = match row.try_into() {
                Ok(source) => source,
                Err(e) => return Err(PostgresExternalServiceError::Deserialize(e))?,
            };
            sources.push(source);
        }

        Ok(sources)
    }

    async fn update_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> anyhow::Result<Source> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresSource::Id,
                PostgresSource::ExternalServiceId,
                PostgresSource::ExternalMetadata,
                PostgresSource::CreatedAt,
                PostgresSource::UpdatedAt,
            ])
            .from(PostgresSource::Table)
            .and_where(Expr::col(PostgresSource::Id).eq(id))
            .lock(LockType::Update)
            .build(PostgresQueryBuilder);

        let row = bind_query_as::<PostgresSourceRow>(sqlx::query_as(&sql), &values)
            .fetch_optional(&mut tx)
            .await?
            .context(SourceError::NotFound(id))?;

        let external_service_id = external_service_id.unwrap_or_else(|| row.external_service_id.into());
        let external_metadata = match external_metadata {
            Some(external_metadata) => PostgresExternalServiceMetadata::try_from(external_metadata)?,
            None => row.external_metadata.0,
        };
        let external_metadata = match serde_json::to_value(&external_metadata) {
            Ok(value) => Some(value),
            Err(e) => return Err(PostgresExternalServiceError::Serialize(e))?,
        };

        let (sql, values) = Query::update()
            .table(PostgresSource::Table)
            .col_expr(PostgresSource::ExternalServiceId, Expr::val(external_service_id).into())
            .col_expr(PostgresSource::ExternalMetadata, Expr::val(external_metadata).into())
            .col_expr(PostgresSource::UpdatedAt, Func::current_timestamp())
            .and_where(Expr::col(PostgresSource::Id).eq(id))
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
            .build(PostgresQueryBuilder);

        let row = bind_query_as::<PostgresSourceRow>(sqlx::query_as(&sql), &values)
            .fetch_one(&mut tx)
            .await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresExternalService::Id,
                PostgresExternalService::Slug,
                PostgresExternalService::Name,
            ])
            .from(PostgresExternalService::Table)
            .and_where(Expr::col(PostgresExternalService::Id).eq(row.external_service_id))
            .build(PostgresQueryBuilder);

        let external_service: ExternalService = bind_query_as::<PostgresExternalServiceRow>(sqlx::query_as(&sql), &values)
            .fetch_one(&mut tx)
            .await?
            .into();

        let source = match Source::try_from((row, external_service)) {
            Ok(source) => {
                source.validate()?;
                source
            },
            Err(e) => return Err(PostgresExternalServiceError::Deserialize(e))?,
        };

        tx.commit().await?;
        Ok(source)
    }

    async fn delete_by_id(&self, id: SourceId) -> anyhow::Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresSource::Table)
            .and_where(Expr::col(PostgresSource::Id).eq(id))
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
    use chrono::NaiveDate;
    use pretty_assertions::{assert_eq, assert_ne};
    use serde_json::json;
    use sqlx::Row;
    use test_context::test_context;
    use uuid::uuid;

    use crate::infrastructure::repository::tests::DatabaseContext;

    use super::*;

    #[test]
    fn convert_fantia() {
        let metadata = PostgresExternalServiceMetadata::Fantia { id: 123456789 };
        let actual = ExternalMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, ExternalMetadata::Fantia { id: 123456789 });

        let metadata = ExternalMetadata::Fantia { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Fantia { id: 123456789 });
    }

    #[test]
    fn convert_nijie() {
        let metadata = PostgresExternalServiceMetadata::Nijie { id: 123456789 };
        let actual = ExternalMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, ExternalMetadata::Nijie { id: 123456789 });

        let metadata = ExternalMetadata::Nijie { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Nijie { id: 123456789 });
    }

    #[test]
    fn convert_pixiv() {
        let metadata = PostgresExternalServiceMetadata::Pixiv { id: 123456789 };
        let actual = ExternalMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, ExternalMetadata::Pixiv { id: 123456789 });

        let metadata = ExternalMetadata::Pixiv { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Pixiv { id: 123456789 });
    }

    #[test]
    fn convert_pixiv_fanbox() {
        let metadata = PostgresExternalServiceMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() };
        let actual = ExternalMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, ExternalMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() });

        let metadata = ExternalMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() });
    }

    #[test]
    fn convert_seiga() {
        let metadata = PostgresExternalServiceMetadata::Seiga { id: 123456789 };
        let actual = ExternalMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, ExternalMetadata::Seiga { id: 123456789 });

        let metadata = ExternalMetadata::Seiga { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Seiga { id: 123456789 });
    }

    #[test]
    fn convert_twitter() {
        let metadata = PostgresExternalServiceMetadata::Twitter { id: 123456789 };
        let actual = ExternalMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, ExternalMetadata::Twitter { id: 123456789 });

        let metadata = ExternalMetadata::Twitter { id: 123456789 };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Twitter { id: 123456789 });
    }

    #[test]
    fn convert_website() {
        let metadata = PostgresExternalServiceMetadata::Website { url: "https://example.com".to_string() };
        let actual = ExternalMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, ExternalMetadata::Website { url: "https://example.com".to_string() });

        let metadata = ExternalMetadata::Website { url: "https://example.com".to_string() };
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Website { url: "https://example.com".to_string() });
    }

    #[test]
    fn convert_custom() {
        let metadata = PostgresExternalServiceMetadata::Custom(json!({ "id": 123456789 }));
        let actual = ExternalMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string()));

        let metadata = ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string());
        let actual = PostgresExternalServiceMetadata::try_from(metadata).unwrap();

        assert_eq!(actual, PostgresExternalServiceMetadata::Custom(json!({ "id": 123456789 })));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresSourcesRepository::new(ctx.pool.clone());
        let actual = repository.create(
            ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            ExternalMetadata::Pixiv { id: 123456789 },
        ).await.unwrap();

        assert_eq!(
            actual.external_service,
            ExternalService {
                id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
        );
        assert_eq!(actual.external_metadata, ExternalMetadata::Pixiv { id: 123456789 });

        let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"));
        assert_eq!(
            actual.get::<serde_json::Value, &str>("external_metadata"),
            json!({
                "type": "pixiv",
                "id": 123456789,
            }),
        );
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_external_metadata_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresSourcesRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_external_metadata(
            ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            ExternalMetadata::Pixiv { id: 8888888 },
        ).await.unwrap();

        assert_eq!(actual, vec![
            Source {
                id: SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 8888888 },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 14),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_with_external_metadata_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresSourcesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")),
            None,
            Some(ExternalMetadata::Pixiv { id: 123456789 }),
        ).await.unwrap();

        assert_eq!(actual.id, SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")));
        assert_eq!(
            actual.external_service,
            ExternalService {
                id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
        );
        assert_eq!(actual.external_metadata, ExternalMetadata::Pixiv { id: 123456789 });
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 14));

        let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
            .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"));
        assert_eq!(
            actual.get::<serde_json::Value, &str>("external_metadata"),
            json!({
                "type": "pixiv",
                "id": 123456789,
            }),
        );
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_with_external_service_and_external_metadata_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresSourcesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")),
            Some(ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7"))),
            Some(ExternalMetadata::Skeb { id: 7777, creator_id: "creator_03".to_string() }),
        ).await.unwrap();

        assert_eq!(actual.id, SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")));
        assert_eq!(
            actual.external_service,
            ExternalService {
                id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                slug: "skeb".to_string(),
                name: "Skeb".to_string(),
            },
        );
        assert_eq!(
            actual.external_metadata,
            ExternalMetadata::Skeb {
                id: 7777,
                creator_id: "creator_03".to_string(),
            },
        );
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 14));

        let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
            .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7"));
        assert_eq!(actual.get::<serde_json::Value, &str>("external_metadata"), json!({
            "type": "skeb",
            "id": 7777,
            "creator_id": "creator_03",
        }));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_fails(ctx: &DatabaseContext) {
        let repository = PostgresSourcesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            None,
            None,
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_succeeds() {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "sources" WHERE "id" = $1"#)
            .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);

        let repository = PostgresSourcesRepository::new(ctx.pool.clone());
        let actual = repository.delete_by_id(SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(1));

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "sources" WHERE "id" = $1"#)
            .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 0);

        let actual = repository.delete_by_id(SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))).await.unwrap();

        assert_eq!(actual, DeleteResult::NotFound);
    }
}
