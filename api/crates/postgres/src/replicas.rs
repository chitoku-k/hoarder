use std::fmt;

use chrono::{DateTime, Utc};
use derive_more::{derive::Display, Constructor, From, Into};
use futures::{future::ready, TryFutureExt, TryStreamExt};
use domain::{
    entity::{
        media::MediumId,
        replicas::{OriginalImage, Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId, ThumbnailImage},
    },
    error::{Error, ErrorKind, Result},
    repository::{replicas::ReplicasRepository, DeleteResult},
};
use sea_query::{Alias, Asterisk, Expr, Iden, JoinType, Keyword, LockType, OnConflict, Order, PostgresQueryBuilder, Query, Value};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool, Row, Type};

use crate::{
    expr::notify::NotifyExpr,
    media::{PostgresMedium, PostgresMediumId},
    sea_query_uuid_value,
};

#[derive(Clone, Constructor)]
pub struct PostgresReplicasRepository {
    pool: PgPool,
}

#[derive(Clone, Debug, From, Into)]
pub(crate) struct PostgresReplicaId(ReplicaId);

#[derive(Clone, Debug, From, Into)]
pub(crate) struct PostgresThumbnailId(ThumbnailId);

#[derive(Debug, FromRow)]
struct PostgresReplicaRow {
    id: PostgresReplicaId,
    medium_id: PostgresMediumId,
    display_order: i32,
    original_url: String,
    mime_type: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    phase: PostgresReplicaPhase,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct PostgresReplicaThumbnailRow {
    replica_id: PostgresReplicaId,
    replica_medium_id: PostgresMediumId,
    replica_display_order: i32,
    replica_original_url: String,
    replica_mime_type: Option<String>,
    replica_width: Option<i32>,
    replica_height: Option<i32>,
    replica_phase: PostgresReplicaPhase,
    replica_created_at: DateTime<Utc>,
    replica_updated_at: DateTime<Utc>,
    thumbnail_id: Option<PostgresThumbnailId>,
    thumbnail_width: Option<i32>,
    thumbnail_height: Option<i32>,
    thumbnail_created_at: Option<DateTime<Utc>>,
    thumbnail_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
struct PostgresThumbnailRow {
    id: PostgresThumbnailId,
    width: i32,
    height: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct PostgresThumbnailDataRow {
    data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PostgresReplicaNotification {
    pub id: ReplicaId,
    pub medium_id: MediumId,
}

#[derive(Iden)]
pub(crate) enum PostgresReplica {
    #[iden = "replicas"]
    Table,
    Id,
    MediumId,
    DisplayOrder,
    OriginalUrl,
    MimeType,
    Width,
    Height,
    Phase,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Display, Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub(crate) enum PostgresReplicaPhase {
    Ready,
    Processing,
    Error,
}

#[derive(Iden)]
pub(crate) enum PostgresThumbnail {
    #[iden = "thumbnails"]
    Table,
    Id,
    ReplicaId,
    Data,
    Width,
    Height,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub(crate) enum PostgresReplicaThumbnail {
    ReplicaId,
    ReplicaMediumId,
    ReplicaDisplayOrder,
    ReplicaOriginalUrl,
    ReplicaMimeType,
    ReplicaWidth,
    ReplicaHeight,
    ReplicaPhase,
    ReplicaCreatedAt,
    ReplicaUpdatedAt,
    ThumbnailId,
    ThumbnailWidth,
    ThumbnailHeight,
    ThumbnailCreatedAt,
    ThumbnailUpdatedAt,
}

#[derive(Iden)]
pub(crate) enum PostgresMediumReplica {
    ReplicaId,
}

sea_query_uuid_value!(PostgresReplicaId, ReplicaId);
sea_query_uuid_value!(PostgresThumbnailId, ThumbnailId);

impl From<PostgresReplicaPhase> for Value {
    fn from(value: PostgresReplicaPhase) -> Self {
        let mut phase = value.to_string();
        phase.make_ascii_lowercase();

        Self::String(Some(Box::new(phase)))
    }
}

impl From<PostgresReplicaRow> for Replica {
    fn from(row: PostgresReplicaRow) -> Self {
        Self {
            id: row.id.into(),
            display_order: row.display_order as u32,
            thumbnail: None,
            original_url: row.original_url,
            mime_type: row.mime_type,
            size: Option::zip(row.width, row.height).map(|(width, height)| Size::new(width as u32, height as u32)),
            status: row.phase.into(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<PostgresReplicaPhase> for ReplicaStatus {
    fn from(value: PostgresReplicaPhase) -> Self {
        use PostgresReplicaPhase::*;
        match value {
            Ready => Self::Ready,
            Processing => Self::Processing,
            Error => Self::Error,
        }
    }
}

impl From<ReplicaStatus> for PostgresReplicaPhase {
    fn from(value: ReplicaStatus) -> Self {
        use ReplicaStatus::*;
        match value {
            Ready => Self::Ready,
            Processing => Self::Processing,
            Error => Self::Error,
        }
    }
}

impl From<PostgresReplicaThumbnailRow> for (MediumId, Replica) {
    fn from(row: PostgresReplicaThumbnailRow) -> Self {
        let thumbnail = {
            if let (
                Some(id),
                Some(width),
                Some(height),
                Some(created_at),
                Some(updated_at),
            ) = (
                row.thumbnail_id,
                row.thumbnail_width,
                row.thumbnail_height,
                row.thumbnail_created_at,
                row.thumbnail_updated_at,
            ) {
                Some(Thumbnail {
                    id: id.into(),
                    size: Size::new(width as u32, height as u32),
                    created_at,
                    updated_at,
                })
            } else {
                None
            }
        };

        (
            row.replica_medium_id.into(),
            Replica {
                id: row.replica_id.into(),
                display_order: row.replica_display_order as u32,
                thumbnail,
                original_url: row.replica_original_url,
                mime_type: row.replica_mime_type,
                size: Option::zip(row.replica_width, row.replica_height).map(|(width, height)| Size::new(width as u32, height as u32)),
                status: row.replica_phase.into(),
                created_at: row.replica_created_at,
                updated_at: row.replica_updated_at,
            },
        )
    }
}

impl From<PostgresThumbnailRow> for Thumbnail {
    fn from(row: PostgresThumbnailRow) -> Self {
        Self {
            id: row.id.into(),
            size: Size::new(row.width as u32, row.height as u32),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<PostgresThumbnailDataRow> for Vec<u8> {
    fn from(row: PostgresThumbnailDataRow) -> Self {
        row.data
    }
}

impl fmt::Display for PostgresReplicaNotification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        json!(self).fmt(f)
    }
}

impl ReplicasRepository for PostgresReplicasRepository {
    #[tracing::instrument(skip_all)]
    async fn create(&self, medium_id: MediumId, thumbnail_image: Option<ThumbnailImage>, original_url: &str, original_image: Option<OriginalImage>, status: ReplicaStatus) -> Result<Replica> {
        let mut tx = self.pool.begin().map_err(Error::other).await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresMedium::Id,
                PostgresMedium::CreatedAt,
                PostgresMedium::UpdatedAt,
            ])
            .from(PostgresMedium::Table)
            .and_where(Expr::col(PostgresMedium::Id).eq(PostgresMediumId::from(medium_id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .fetch_all(&mut *tx)
            .await
            .map_err(Error::other)?;

        let (sql, values) = Query::select()
            .expr(
                Expr::col(Asterisk)
                    .count()
                    .add(Expr::val(1i32)),
            )
            .from(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::MediumId).eq(PostgresMediumId::from(medium_id)))
            .build_sqlx(PostgresQueryBuilder);

        let order: i64 = sqlx::query_with(&sql, values)
            .fetch_one(&mut *tx)
            .await
            .and_then(|r| r.try_get(0))
            .map_err(Error::other)?;

        let (sql, values) = Query::insert()
            .into_table(PostgresReplica::Table)
            .columns([
                PostgresReplica::MediumId,
                PostgresReplica::DisplayOrder,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
                PostgresReplica::Width,
                PostgresReplica::Height,
                PostgresReplica::Phase,
            ])
            .values([
                PostgresMediumId::from(medium_id).into(),
                order.into(),
                original_url.into(),
                original_image.as_ref().map(|original_image| original_image.mime_type).into(),
                original_image.as_ref().map(|original_image| original_image.size.width).into(),
                original_image.as_ref().map(|original_image| original_image.size.height).into(),
                PostgresReplicaPhase::from(status).into(),
            ])
            .map_err(Error::other)?
            .returning(
                Query::returning()
                    .exprs([
                        Expr::col(PostgresReplica::Id),
                        Expr::col(PostgresReplica::MediumId),
                        Expr::col(PostgresReplica::DisplayOrder),
                        Expr::col(PostgresReplica::OriginalUrl),
                        Expr::col(PostgresReplica::MimeType),
                        Expr::col(PostgresReplica::Width),
                        Expr::col(PostgresReplica::Height),
                        Expr::col(PostgresReplica::Phase),
                        Expr::col(PostgresReplica::CreatedAt),
                        Expr::col(PostgresReplica::UpdatedAt),
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let mut replica = match sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => Replica::from(row),
            Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::MediumNotFound { id: medium_id })?,
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => return Err(ErrorKind::ReplicaOriginalUrlDuplicate { original_url: original_url.to_string(), entry: None })?,
            Err(e) => return Err(Error::other(e)),
        };

        if let Some(thumbnail_image) = thumbnail_image {
            let (sql, values) = Query::insert()
                .into_table(PostgresThumbnail::Table)
                .columns([
                    PostgresThumbnail::ReplicaId,
                    PostgresThumbnail::Data,
                    PostgresThumbnail::Width,
                    PostgresThumbnail::Height,
                ])
                .values([
                    PostgresReplicaId::from(replica.id).into(),
                    thumbnail_image.body.into(),
                    thumbnail_image.size.width.into(),
                    thumbnail_image.size.height.into(),
                ])
                .map_err(Error::other)?
                .returning(
                    Query::returning()
                        .exprs([
                            Expr::col(PostgresThumbnail::Id),
                            Expr::col(PostgresThumbnail::Width),
                            Expr::col(PostgresThumbnail::Height),
                            Expr::col(PostgresThumbnail::CreatedAt),
                            Expr::col(PostgresThumbnail::UpdatedAt),
                        ])
                )
                .build_sqlx(PostgresQueryBuilder);

            let thumbnail = sqlx::query_as_with::<_, PostgresThumbnailRow, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(Error::other)?
                .into();

            replica.thumbnail = Some(thumbnail);
        }

        let (sql, values) = Query::select()
            .expr(NotifyExpr::notify(PostgresReplica::Table.to_string(), PostgresReplicaNotification { id: replica.id, medium_id }))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *tx).await.map_err(Error::other)?;

        tx.commit().await.map_err(Error::other)?;
        Ok(replica)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_ids<T>(&self, ids: T) -> Result<Vec<Replica>>
    where
        T: Iterator<Item = ReplicaId> + Send,
    {
        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Id)), PostgresReplicaThumbnail::ReplicaId)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::MediumId)), PostgresReplicaThumbnail::ReplicaMediumId)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::DisplayOrder)), PostgresReplicaThumbnail::ReplicaDisplayOrder)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::OriginalUrl)), PostgresReplicaThumbnail::ReplicaOriginalUrl)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::MimeType)), PostgresReplicaThumbnail::ReplicaMimeType)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Width)), PostgresReplicaThumbnail::ReplicaWidth)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Height)), PostgresReplicaThumbnail::ReplicaHeight)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Phase)), PostgresReplicaThumbnail::ReplicaPhase)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::CreatedAt)), PostgresReplicaThumbnail::ReplicaCreatedAt)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::UpdatedAt)), PostgresReplicaThumbnail::ReplicaUpdatedAt)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::Id)), PostgresReplicaThumbnail::ThumbnailId)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::Width)), PostgresReplicaThumbnail::ThumbnailWidth)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::Height)), PostgresReplicaThumbnail::ThumbnailHeight)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::CreatedAt)), PostgresReplicaThumbnail::ThumbnailCreatedAt)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::UpdatedAt)), PostgresReplicaThumbnail::ThumbnailUpdatedAt)
            .from(PostgresReplica::Table)
            .join(
                JoinType::LeftJoin,
                PostgresThumbnail::Table,
                Expr::col((PostgresReplica::Table, PostgresReplica::Id))
                    .equals((PostgresThumbnail::Table, PostgresThumbnail::ReplicaId)),
            )
            .and_where(Expr::col((PostgresReplica::Table, PostgresReplica::Id)).is_in(ids.map(PostgresReplicaId::from)))
            .order_by(PostgresReplica::MediumId, Order::Asc)
            .order_by(PostgresReplica::DisplayOrder, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let replicas = sqlx::query_as_with::<_, PostgresReplicaThumbnailRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .map_ok(|(_, replica)| replica)
            .try_collect()
            .await
            .map_err(Error::other)?;

        Ok(replicas)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_original_url(&self, original_url: &str) -> Result<Replica> {
        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Id)), PostgresReplicaThumbnail::ReplicaId)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::MediumId)), PostgresReplicaThumbnail::ReplicaMediumId)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::DisplayOrder)), PostgresReplicaThumbnail::ReplicaDisplayOrder)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::OriginalUrl)), PostgresReplicaThumbnail::ReplicaOriginalUrl)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::MimeType)), PostgresReplicaThumbnail::ReplicaMimeType)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Width)), PostgresReplicaThumbnail::ReplicaWidth)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Height)), PostgresReplicaThumbnail::ReplicaHeight)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Phase)), PostgresReplicaThumbnail::ReplicaPhase)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::CreatedAt)), PostgresReplicaThumbnail::ReplicaCreatedAt)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::UpdatedAt)), PostgresReplicaThumbnail::ReplicaUpdatedAt)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::Id)), PostgresReplicaThumbnail::ThumbnailId)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::Width)), PostgresReplicaThumbnail::ThumbnailWidth)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::Height)), PostgresReplicaThumbnail::ThumbnailHeight)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::CreatedAt)), PostgresReplicaThumbnail::ThumbnailCreatedAt)
            .expr_as(Expr::col((PostgresThumbnail::Table, PostgresThumbnail::UpdatedAt)), PostgresReplicaThumbnail::ThumbnailUpdatedAt)
            .from(PostgresReplica::Table)
            .join(
                JoinType::LeftJoin,
                PostgresThumbnail::Table,
                Expr::col((PostgresReplica::Table, PostgresReplica::Id))
                    .equals((PostgresThumbnail::Table, PostgresThumbnail::ReplicaId)),
            )
            .and_where(Expr::col(PostgresReplica::OriginalUrl).eq(original_url))
            .build_sqlx(PostgresQueryBuilder);

        let (_, replica) = match sqlx::query_as_with::<_, PostgresReplicaThumbnailRow, _>(&sql, values).fetch_one(&self.pool).await {
            Ok(row) => row.into(),
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::ReplicaNotFoundByUrl { original_url: original_url.to_string() })?,
            Err(e) => return Err(Error::other(e)),
        };

        Ok(replica)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_thumbnail_by_id(&self, id: ThumbnailId) -> Result<Vec<u8>> {
        let (sql, values) = Query::select()
            .columns([
                PostgresThumbnail::Data,
            ])
            .from(PostgresThumbnail::Table)
            .and_where(Expr::col(PostgresThumbnail::Id).eq(PostgresThumbnailId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        let thumbnail = match sqlx::query_as_with::<_, PostgresThumbnailDataRow, _>(&sql, values).fetch_one(&self.pool).await {
            Ok(row) => row.into(),
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::ThumbnailNotFound { id })?,
            Err(e) => return Err(Error::other(e)),
        };

        Ok(thumbnail)
    }

    #[tracing::instrument(skip_all)]
    async fn update_by_id(&self, id: ReplicaId, thumbnail_image: Option<Option<ThumbnailImage>>, original_url: Option<&str>, original_image: Option<Option<OriginalImage>>, status: Option<ReplicaStatus>) -> Result<Replica> {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresReplica::Id,
                PostgresReplica::MediumId,
                PostgresReplica::DisplayOrder,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
                PostgresReplica::Width,
                PostgresReplica::Height,
                PostgresReplica::Phase,
                PostgresReplica::CreatedAt,
                PostgresReplica::UpdatedAt,
            ])
            .from(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let medium_id = match sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => MediumId::from(row.medium_id),
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::ReplicaNotFound { id })?,
            Err(e) => return Err(Error::other(e)),
        };

        let mut query = Query::update();
        query
            .table(PostgresReplica::Table)
            .value(PostgresReplica::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .returning(
                Query::returning()
                    .exprs([
                        Expr::col(PostgresReplica::Id),
                        Expr::col(PostgresReplica::MediumId),
                        Expr::col(PostgresReplica::DisplayOrder),
                        Expr::col(PostgresReplica::OriginalUrl),
                        Expr::col(PostgresReplica::MimeType),
                        Expr::col(PostgresReplica::Width),
                        Expr::col(PostgresReplica::Height),
                        Expr::col(PostgresReplica::Phase),
                        Expr::col(PostgresReplica::CreatedAt),
                        Expr::col(PostgresReplica::UpdatedAt),
                    ])
            );

        if let Some(original_url) = original_url {
            query.value(PostgresReplica::OriginalUrl, original_url);
        }
        if let Some(original_image) = original_image {
            let (mime_type, width, height) = match original_image {
                Some(original_image) => (Some(original_image.mime_type), Some(original_image.size.width), Some(original_image.size.height)),
                None => (None, None, None),
            };
            query.value(PostgresReplica::MimeType, mime_type);
            query.value(PostgresReplica::Width, width);
            query.value(PostgresReplica::Height, height);
        }
        if let Some(status) = status {
            query.value(PostgresReplica::Phase, PostgresReplicaPhase::from(status));
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let mut replica = match sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => Replica::from(row),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                if let Some(original_url) = original_url {
                    return Err(ErrorKind::ReplicaOriginalUrlDuplicate { original_url: original_url.to_string(), entry: None })?;
                }
                return Err(Error::other(e));
            },
            Err(e) => return Err(Error::other(e)),
        };

        if let Some(thumbnail_image) = thumbnail_image {
            let (body, width, height) = match thumbnail_image {
                Some(thumbnail_image) => (Some(thumbnail_image.body), Some(thumbnail_image.size.width), Some(thumbnail_image.size.height)),
                None => (None, None, None),
            };
            let (sql, values) = Query::insert()
                .into_table(PostgresThumbnail::Table)
                .columns([
                    PostgresThumbnail::ReplicaId,
                    PostgresThumbnail::Data,
                    PostgresThumbnail::Width,
                    PostgresThumbnail::Height,
                ])
                .values([
                    PostgresReplicaId::from(replica.id).into(),
                    body.into(),
                    width.into(),
                    height.into(),
                ])
                .map_err(Error::other)?
                .on_conflict(
                    OnConflict::column(PostgresThumbnail::ReplicaId)
                        .update_columns([
                            PostgresThumbnail::Data,
                            PostgresThumbnail::Width,
                            PostgresThumbnail::Height,
                        ])
                        .value(PostgresThumbnail::UpdatedAt, Expr::current_timestamp())
                        .to_owned()
                )
                .returning(
                    Query::returning()
                        .exprs([
                            Expr::col(PostgresThumbnail::Id),
                            Expr::col(PostgresThumbnail::Width),
                            Expr::col(PostgresThumbnail::Height),
                            Expr::col(PostgresThumbnail::CreatedAt),
                            Expr::col(PostgresThumbnail::UpdatedAt),
                        ])
                )
                .build_sqlx(PostgresQueryBuilder);

            let thumbnail = match sqlx::query_as_with::<_, PostgresThumbnailRow, _>(&sql, values).fetch_one(&mut *tx).await {
                Ok(row) => row.into(),
                Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::ReplicaNotFound { id })?,
                Err(e) => return Err(Error::other(e)),
            };

            replica.thumbnail = Some(thumbnail);
        }

        let (sql, values) = Query::select()
            .expr(NotifyExpr::notify(PostgresReplica::Table.to_string(), PostgresReplicaNotification { id: replica.id, medium_id }))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *tx).await.map_err(Error::other)?;

        tx.commit().await.map_err(Error::other)?;
        Ok(replica)
    }

    #[tracing::instrument(skip_all)]
    async fn delete_by_id(&self, id: ReplicaId) -> Result<DeleteResult> {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let siblings = Alias::new("siblings");
        let (sql, values) = Query::select()
            .columns([
                (siblings.clone(), PostgresReplica::Id),
                (siblings.clone(), PostgresReplica::MediumId),
                (siblings.clone(), PostgresReplica::DisplayOrder),
                (siblings.clone(), PostgresReplica::OriginalUrl),
                (siblings.clone(), PostgresReplica::MimeType),
                (siblings.clone(), PostgresReplica::Width),
                (siblings.clone(), PostgresReplica::Height),
                (siblings.clone(), PostgresReplica::Phase),
                (siblings.clone(), PostgresReplica::CreatedAt),
                (siblings.clone(), PostgresReplica::UpdatedAt),
            ])
            .from(PostgresReplica::Table)
            .join_as(
                JoinType::InnerJoin,
                PostgresReplica::Table,
                siblings.clone(),
                Expr::col((siblings.clone(), PostgresReplica::MediumId))
                    .equals((PostgresReplica::Table, PostgresReplica::MediumId)),
            )
            .and_where(Expr::col((PostgresReplica::Table, PostgresReplica::Id)).eq(PostgresReplicaId::from(id)))
            .order_by((siblings.clone(), PostgresReplica::DisplayOrder), Order::Asc)
            .lock_with_tables(LockType::Update, [siblings])
            .build_sqlx(PostgresQueryBuilder);

        let siblings: Vec<Replica> = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch(&mut *tx)
            .map_ok(Replica::from)
            .try_filter(|r| ready(r.id != id))
            .try_collect()
            .await
            .map_err(Error::other)?;

        let (sql, values) = Query::delete()
            .from_table(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        let affected = sqlx::query_with(&sql, values)
            .execute(&mut *tx)
            .await
            .map_err(Error::other)?
            .rows_affected();

        let result = match affected {
            0 => return Ok(DeleteResult::NotFound),
            count => DeleteResult::Deleted(count),
        };

        let (sql, values) = Query::update()
            .table(PostgresReplica::Table)
            .value(PostgresReplica::DisplayOrder, Keyword::Null)
            .and_where(Expr::col(PostgresReplica::Id).is_in(siblings.iter().map(|s| *s.id)))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *tx)
            .await
            .map_err(Error::other)?;

        for (order, sibling) in siblings.into_iter().enumerate() {
            let (sql, values) = Query::update()
                .table(PostgresReplica::Table)
                .value(PostgresReplica::DisplayOrder, Expr::val(order as i32 + 1))
                .value(PostgresReplica::UpdatedAt, Expr::current_timestamp())
                .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(sibling.id)))
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(Error::other)?;
        }

        tx.commit().await.map_err(Error::other)?;
        Ok(result)
    }
}
