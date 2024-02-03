use anyhow::Context;
use chrono::{DateTime, Utc};
use derive_more::{Constructor, From, Into};
use futures::{future::ready, TryStreamExt};
use domain::{
    entity::{
        media::MediumId,
        replicas::{OriginalImage, Replica, ReplicaError, ReplicaId, Size, Thumbnail, ThumbnailError, ThumbnailId, ThumbnailImage},
    },
    repository::{replicas::ReplicasRepository, DeleteResult},
};
use sea_query::{Alias, Asterisk, Expr, Iden, JoinType, Keyword, LockType, OnConflict, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, PgPool, Row};

use crate::{
    media::PostgresMediumId,
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
    display_order: i32,
    original_url: String,
    mime_type: String,
    width: i32,
    height: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct PostgresReplicaThumbnailRow {
    replica_id: PostgresReplicaId,
    replica_medium_id: PostgresMediumId,
    replica_display_order: i32,
    replica_original_url: String,
    replica_mime_type: String,
    replica_width: i32,
    replica_height: i32,
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
    CreatedAt,
    UpdatedAt,
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

impl From<PostgresReplicaRow> for Replica {
    fn from(row: PostgresReplicaRow) -> Self {
        Self {
            id: row.id.into(),
            display_order: row.display_order as u32,
            thumbnail: None,
            original_url: row.original_url,
            mime_type: row.mime_type,
            size: Size::new(row.width as u32, row.height as u32),
            created_at: row.created_at,
            updated_at: row.updated_at,
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
                size: Size::new(row.replica_width as u32, row.replica_height as u32),
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

impl ReplicasRepository for PostgresReplicasRepository {
    async fn create(&self, medium_id: MediumId, thumbnail_image: Option<ThumbnailImage>, original_url: &str, original_image: OriginalImage) -> anyhow::Result<Replica> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresReplica::Id,
            ])
            .from(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::MediumId).eq(PostgresMediumId::from(medium_id)))
            .order_by(PostgresReplica::Id, Order::Asc)
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .fetch_all(&mut *tx)
            .await?;

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
            .await?
            .try_get(0)?;

        let (sql, values) = Query::insert()
            .into_table(PostgresReplica::Table)
            .columns([
                PostgresReplica::MediumId,
                PostgresReplica::DisplayOrder,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
                PostgresReplica::Width,
                PostgresReplica::Height,
            ])
            .values([
                PostgresMediumId::from(medium_id).into(),
                order.into(),
                original_url.into(),
                original_image.mime_type.into(),
                original_image.size.width.into(),
                original_image.size.height.into(),
            ])?
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
                        Expr::col(PostgresReplica::CreatedAt),
                        Expr::col(PostgresReplica::UpdatedAt),
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let mut replica: Replica = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch_one(&mut *tx)
            .await?
            .into();

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
                ])?
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
                .await?
                .into();

            replica.thumbnail = Some(thumbnail);
        }

        tx.commit().await?;
        Ok(replica)
    }

    async fn fetch_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<Replica>>
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static,
    {
        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Id)), PostgresReplicaThumbnail::ReplicaId)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::MediumId)), PostgresReplicaThumbnail::ReplicaMediumId)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::DisplayOrder)), PostgresReplicaThumbnail::ReplicaDisplayOrder)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::OriginalUrl)), PostgresReplicaThumbnail::ReplicaOriginalUrl)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::MimeType)), PostgresReplicaThumbnail::ReplicaMimeType)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Width)), PostgresReplicaThumbnail::ReplicaWidth)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Height)), PostgresReplicaThumbnail::ReplicaHeight)
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
            .and_where(Expr::col((PostgresReplica::Table, PostgresReplica::Id)).is_in(ids.into_iter().map(PostgresReplicaId::from)))
            .order_by(PostgresReplica::MediumId, Order::Asc)
            .order_by(PostgresReplica::DisplayOrder, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let replicas = sqlx::query_as_with::<_, PostgresReplicaThumbnailRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .map_ok(|(_, replica)| replica)
            .try_collect()
            .await?;

        Ok(replicas)
    }

    async fn fetch_by_original_url(&self, original_url: &str) -> anyhow::Result<Replica> {
        let (sql, values) = Query::select()
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Id)), PostgresReplicaThumbnail::ReplicaId)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::MediumId)), PostgresReplicaThumbnail::ReplicaMediumId)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::DisplayOrder)), PostgresReplicaThumbnail::ReplicaDisplayOrder)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::OriginalUrl)), PostgresReplicaThumbnail::ReplicaOriginalUrl)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::MimeType)), PostgresReplicaThumbnail::ReplicaMimeType)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Width)), PostgresReplicaThumbnail::ReplicaWidth)
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Height)), PostgresReplicaThumbnail::ReplicaHeight)
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

        let (_, replica) = sqlx::query_as_with::<_, PostgresReplicaThumbnailRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?
            .context(ReplicaError::NotFoundByURL(original_url.to_string()))?
            .into();

        Ok(replica)
    }

    async fn fetch_thumbnail_by_id(&self, id: ThumbnailId) -> anyhow::Result<Vec<u8>> {
        let (sql, values) = Query::select()
            .columns([
                PostgresThumbnail::Data,
            ])
            .from(PostgresThumbnail::Table)
            .and_where(Expr::col(PostgresThumbnail::Id).eq(PostgresThumbnailId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        let thumbnail = sqlx::query_as_with::<_, PostgresThumbnailDataRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?
            .context(ThumbnailError::NotFoundById(id))?
            .into();

        Ok(thumbnail)
    }

    async fn update_by_id<'a>(&self, id: ReplicaId, thumbnail_image: Option<ThumbnailImage>, original_url: Option<&'a str>, original_image: Option<OriginalImage>) -> anyhow::Result<Replica> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresReplica::Id,
                PostgresReplica::DisplayOrder,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
                PostgresReplica::Width,
                PostgresReplica::Height,
                PostgresReplica::CreatedAt,
                PostgresReplica::UpdatedAt,
            ])
            .from(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch_optional(&mut *tx)
            .await?
            .context(ReplicaError::NotFoundById(id))?;

        let mut query = Query::update();
        query
            .table(PostgresReplica::Table)
            .value(PostgresReplica::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .returning(
                Query::returning()
                    .exprs([
                        Expr::col(PostgresReplica::Id),
                        Expr::col(PostgresReplica::DisplayOrder),
                        Expr::col(PostgresReplica::OriginalUrl),
                        Expr::col(PostgresReplica::MimeType),
                        Expr::col(PostgresReplica::Width),
                        Expr::col(PostgresReplica::Height),
                        Expr::col(PostgresReplica::CreatedAt),
                        Expr::col(PostgresReplica::UpdatedAt),
                    ])
            );

        if let Some(original_url) = original_url {
            query.value(PostgresReplica::OriginalUrl, original_url);
        }
        if let Some(original_image) = original_image {
            query.value(PostgresReplica::MimeType, original_image.mime_type);
            query.value(PostgresReplica::Width, original_image.size.width);
            query.value(PostgresReplica::Height, original_image.size.height);
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let mut replica: Replica = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch_one(&mut *tx)
            .await?
            .into();

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
                ])?
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

            let thumbnail = sqlx::query_as_with::<_, PostgresThumbnailRow, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await?
                .into();

            replica.thumbnail = Some(thumbnail);
        }

        tx.commit().await?;
        Ok(replica)
    }

    async fn delete_by_id(&self, id: ReplicaId) -> anyhow::Result<DeleteResult> {
        let mut tx = self.pool.begin().await?;

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
            .await?;

        let (sql, values) = Query::delete()
            .from_table(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        let affected = sqlx::query_with(&sql, values)
            .execute(&mut *tx)
            .await?
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

        sqlx::query_with(&sql, values).execute(&mut *tx).await?;

        for (order, sibling) in siblings.into_iter().enumerate() {
            let (sql, values) = Query::update()
                .table(PostgresReplica::Table)
                .value(PostgresReplica::DisplayOrder, Expr::val(order as i32 + 1))
                .value(PostgresReplica::UpdatedAt, Expr::current_timestamp())
                .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(sibling.id)))
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(result)
    }
}
