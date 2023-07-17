use anyhow::Context;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use derive_more::{Constructor, From, Into};
use futures::TryStreamExt;
use domain::{
    entity::{
        media::MediumId,
        replicas::{Replica, ReplicaError, ReplicaId, ReplicaThumbnail},
    },
    repository::{replicas::ReplicasRepository, DeleteResult},
};
use sea_query::{Alias, Asterisk, BinOper, Expr, Iden, JoinType, Keyword, LockType, Order, PostgresQueryBuilder, Query};
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

#[derive(Debug, FromRow)]
pub(crate) struct PostgresReplicaRow {
    id: PostgresReplicaId,
    medium_id: PostgresMediumId,
    display_order: Option<i32>,
    has_thumbnail: bool,
    original_url: String,
    mime_type: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, FromRow)]
struct PostgresReplicaThumbnailRow {
    id: PostgresReplicaId,
    display_order: Option<i32>,
    thumbnail: Option<Vec<u8>>,
    original_url: String,
    mime_type: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Iden)]
pub(crate) enum PostgresReplica {
    #[iden = "replicas"]
    Table,
    Id,
    MediumId,
    DisplayOrder,
    HasThumbnail,
    Thumbnail,
    OriginalUrl,
    MimeType,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub(crate) enum PostgresMediumReplica {
    ReplicaId,
}

sea_query_uuid_value!(PostgresReplicaId, ReplicaId);

impl From<PostgresReplicaRow> for Replica {
    fn from(row: PostgresReplicaRow) -> Self {
        Self {
            id: row.id.into(),
            display_order: row.display_order.map(|o| o as u32),
            has_thumbnail: row.has_thumbnail,
            original_url: row.original_url,
            mime_type: row.mime_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<PostgresReplicaRow> for (MediumId, Replica) {
    fn from(row: PostgresReplicaRow) -> Self {
        (
            row.medium_id.into(),
            Replica {
                id: row.id.into(),
                display_order: row.display_order.map(|o| o as u32),
                has_thumbnail: row.has_thumbnail,
                original_url: row.original_url,
                mime_type: row.mime_type,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
        )
    }
}

impl From<PostgresReplicaThumbnailRow> for ReplicaThumbnail {
    fn from(row: PostgresReplicaThumbnailRow) -> Self {
        Self {
            id: row.id.into(),
            display_order: row.display_order.map(|o| o as u32),
            thumbnail: row.thumbnail.map(Into::into),
            original_url: row.original_url,
            mime_type: row.mime_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl ReplicasRepository for PostgresReplicasRepository {
    async fn create(&self, medium_id: MediumId, thumbnail: Option<Vec<u8>>, original_url: &str, mime_type: &str) -> anyhow::Result<Replica> {
        let mut tx = self.pool.begin().await?;

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
            .fetch_one(&mut tx)
            .await?
            .try_get(0)?;

        let (sql, values) = Query::insert()
            .into_table(PostgresReplica::Table)
            .columns([
                PostgresReplica::MediumId,
                PostgresReplica::DisplayOrder,
                PostgresReplica::Thumbnail,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
            ])
            .values([
                PostgresMediumId::from(medium_id).into(),
                order.into(),
                thumbnail.into(),
                original_url.into(),
                mime_type.into(),
            ])?
            .returning(
                Query::returning()
                    .exprs([
                        Expr::expr(Expr::col(PostgresReplica::Thumbnail).is_not_null())
                            .binary(BinOper::As, Expr::col(PostgresReplica::HasThumbnail)),
                        Expr::col(PostgresReplica::Id).into(),
                        Expr::col(PostgresReplica::MediumId).into(),
                        Expr::col(PostgresReplica::DisplayOrder).into(),
                        Expr::col(PostgresReplica::OriginalUrl).into(),
                        Expr::col(PostgresReplica::MimeType).into(),
                        Expr::col(PostgresReplica::CreatedAt).into(),
                        Expr::col(PostgresReplica::UpdatedAt).into(),
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let replica = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch_one(&mut tx)
            .await?
            .into();

        tx.commit().await?;
        Ok(replica)
    }

    async fn fetch_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<Replica>>
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static,
    {
        let (sql, values) = Query::select()
            .expr_as(
                Expr::col(PostgresReplica::Thumbnail).is_not_null(),
                PostgresReplica::HasThumbnail,
            )
            .columns([
                PostgresReplica::Id,
                PostgresReplica::MediumId,
                PostgresReplica::DisplayOrder,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
                PostgresReplica::CreatedAt,
                PostgresReplica::UpdatedAt,
            ])
            .from(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Id).is_in(ids.into_iter().map(PostgresReplicaId::from)))
            .order_by(PostgresReplica::MediumId, Order::Asc)
            .order_by(PostgresReplica::DisplayOrder, Order::Asc)
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let replicas = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .try_collect()
            .await?;

        Ok(replicas)
    }

    async fn fetch_by_original_url(&self, original_url: &str) -> anyhow::Result<Replica> {
        let (sql, values) = Query::select()
            .expr_as(
                Expr::col(PostgresReplica::Thumbnail).is_not_null(),
                PostgresReplica::HasThumbnail,
            )
            .columns([
                PostgresReplica::Id,
                PostgresReplica::MediumId,
                PostgresReplica::DisplayOrder,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
                PostgresReplica::CreatedAt,
                PostgresReplica::UpdatedAt,
            ])
            .from(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::OriginalUrl).eq(original_url))
            .build_sqlx(PostgresQueryBuilder);

        let replica = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?
            .map(Into::into)
            .context(ReplicaError::NotFoundByURL(original_url.to_string()))?;

        Ok(replica)
    }

    async fn fetch_thumbnail_by_id(&self, id: ReplicaId) -> anyhow::Result<ReplicaThumbnail> {
        let (sql, values) = Query::select()
            .columns([
                PostgresReplica::Id,
                PostgresReplica::DisplayOrder,
                PostgresReplica::Thumbnail,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
                PostgresReplica::CreatedAt,
                PostgresReplica::UpdatedAt,
            ])
            .from(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        let thumbnail = sqlx::query_as_with::<_, PostgresReplicaThumbnailRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?
            .map(Into::into)
            .context(ReplicaError::NotFoundById(id))?;

        Ok(thumbnail)
    }

    async fn update_by_id<'a>(&self, id: ReplicaId, thumbnail: Option<Vec<u8>>, original_url: Option<&'a str>, mime_type: Option<&'a str>) -> anyhow::Result<Replica> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::select()
            .expr_as(
                Expr::col(PostgresReplica::Thumbnail).is_not_null(),
                PostgresReplica::HasThumbnail,
            )
            .columns([
                PostgresReplica::Id,
                PostgresReplica::MediumId,
                PostgresReplica::DisplayOrder,
                PostgresReplica::OriginalUrl,
                PostgresReplica::MimeType,
                PostgresReplica::CreatedAt,
                PostgresReplica::UpdatedAt,
            ])
            .from(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch_optional(&mut tx)
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
                        Expr::expr(Expr::col(PostgresReplica::Thumbnail).is_not_null())
                            .binary(BinOper::As, Expr::col(PostgresReplica::HasThumbnail)),
                        Expr::col(PostgresReplica::Id).into(),
                        Expr::col(PostgresReplica::MediumId).into(),
                        Expr::col(PostgresReplica::DisplayOrder).into(),
                        Expr::col(PostgresReplica::OriginalUrl).into(),
                        Expr::col(PostgresReplica::MimeType).into(),
                        Expr::col(PostgresReplica::CreatedAt).into(),
                        Expr::col(PostgresReplica::UpdatedAt).into(),
                    ])
            );

        if let Some(thumbnail) = thumbnail {
            query.value(PostgresReplica::Thumbnail, thumbnail);
        }
        if let Some(original_url) = original_url {
            query.value(PostgresReplica::OriginalUrl, original_url);
        }
        if let Some(mime_type) = mime_type {
            query.value(PostgresReplica::MimeType, mime_type);
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let replica = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch_one(&mut tx)
            .await?
            .into();

        tx.commit().await?;
        Ok(replica)
    }

    async fn delete_by_id(&self, id: ReplicaId) -> anyhow::Result<DeleteResult> {
        let mut tx = self.pool.begin().await?;

        let siblings = Alias::new("siblings");
        let (sql, values) = Query::select()
            .expr_as(
                Expr::col((siblings.clone(), PostgresReplica::Thumbnail)).is_not_null(),
                PostgresReplica::HasThumbnail,
            )
            .columns([
                (siblings.clone(), PostgresReplica::Id),
                (siblings.clone(), PostgresReplica::MediumId),
                (siblings.clone(), PostgresReplica::DisplayOrder),
                (siblings.clone(), PostgresReplica::OriginalUrl),
                (siblings.clone(), PostgresReplica::MimeType),
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
            .and_where(Expr::col((siblings.clone(), PostgresReplica::Id)).ne(PostgresReplicaId::from(id)))
            .order_by((siblings, PostgresReplica::DisplayOrder), Order::Asc)
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let siblings: Vec<Replica> = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values)
            .fetch(&mut tx)
            .map_ok(Into::into)
            .try_collect()
            .await?;

        let (sql, values) = Query::delete()
            .from_table(PostgresReplica::Table)
            .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        let affected = sqlx::query_with(&sql, values)
            .execute(&mut tx)
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

        sqlx::query_with(&sql, values).execute(&mut tx).await?;

        for (order, sibling) in siblings.into_iter().enumerate() {
            let (sql, values) = Query::update()
                .table(PostgresReplica::Table)
                .value(PostgresReplica::DisplayOrder, Expr::val(order as i32 + 1))
                .value(PostgresReplica::UpdatedAt, Expr::current_timestamp())
                .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(sibling.id)))
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut tx).await?;
        }

        tx.commit().await?;
        Ok(result)
    }
}
