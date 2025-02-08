use derive_more::{Constructor, From, Into};
use domain::{
    entity::tag_types::{TagType, TagTypeId},
    error::{Error, ErrorKind, Result},
    repository::{tag_types::TagTypesRepository, DeleteResult},
};
use futures::TryStreamExt;
use sea_query::{Expr, Iden, LockType, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, PgPool};

use crate::sea_query_uuid_value;

#[derive(Clone, Constructor)]
pub struct PostgresTagTypesRepository {
    pool: PgPool,
}

#[derive(Clone, Debug, From, Into)]
pub(crate) struct PostgresTagTypeId(TagTypeId);

#[derive(Debug, FromRow)]
struct PostgresTagTypeRow {
    id: PostgresTagTypeId,
    slug: String,
    name: String,
    kana: String,
}

#[derive(Iden)]
pub(crate) enum PostgresTagType {
    #[iden = "tag_types"]
    Table,
    Id,
    Slug,
    Name,
    Kana,
}

#[derive(Iden)]
pub(crate) enum PostgresTagTagType {
    TagTypeId,
    TagTypeSlug,
    TagTypeName,
    TagTypeKana,
}

sea_query_uuid_value!(PostgresTagTypeId, TagTypeId);

impl From<PostgresTagTypeRow> for TagType {
    fn from(row: PostgresTagTypeRow) -> Self {
        Self {
            id: row.id.into(),
            slug: row.slug,
            name: row.name,
            kana: row.kana,
        }
    }
}

impl TagTypesRepository for PostgresTagTypesRepository {
    #[tracing::instrument(skip_all)]
    async fn create(&self, slug: &str, name: &str, kana: &str) -> Result<TagType> {
        let (sql, values) = Query::insert()
            .into_table(PostgresTagType::Table)
            .columns([PostgresTagType::Slug, PostgresTagType::Name, PostgresTagType::Kana])
            .values([Expr::val(slug).into(), Expr::val(name).into(), Expr::val(kana).into()])
            .map_err(Error::other)?
            .returning(
                Query::returning()
                    .columns([
                        PostgresTagType::Id,
                        PostgresTagType::Slug,
                        PostgresTagType::Name,
                        PostgresTagType::Kana,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let tag_type = match sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values).fetch_one(&self.pool).await {
            Ok(row) => row.into(),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => return Err(ErrorKind::TagTypeSlugDuplicate { slug: slug.to_string() })?,
            Err(e) => return Err(Error::other(e)),
        };

        Ok(tag_type)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_ids<T>(&self, ids: T) -> Result<Vec<TagType>>
    where
        T: Iterator<Item = TagTypeId> + Send,
    {
        let (sql, values) = Query::select()
            .columns([
                PostgresTagType::Id,
                PostgresTagType::Slug,
                PostgresTagType::Name,
                PostgresTagType::Kana,
            ])
            .from(PostgresTagType::Table)
            .and_where(Expr::col(PostgresTagType::Id).is_in(ids.map(PostgresTagTypeId::from)))
            .order_by(PostgresTagType::Kana, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let tag_types = sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        Ok(tag_types)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_all(&self) -> Result<Vec<TagType>> {
        let (sql, values) = Query::select()
            .columns([
                PostgresTagType::Id,
                PostgresTagType::Slug,
                PostgresTagType::Name,
                PostgresTagType::Kana,
            ])
            .from(PostgresTagType::Table)
            .order_by(PostgresTagType::Kana, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let tag_types = sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        Ok(tag_types)
    }

    #[tracing::instrument(skip_all)]
    async fn update_by_id(&self, id: TagTypeId, slug: Option<&str>, name: Option<&str>, kana: Option<&str>) -> Result<TagType> {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresTagType::Id,
                PostgresTagType::Slug,
                PostgresTagType::Name,
                PostgresTagType::Kana,
            ])
            .from(PostgresTagType::Table)
            .and_where(Expr::col(PostgresTagType::Id).eq(PostgresTagTypeId::from(id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let tag_type = match sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => TagType::from(row),
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::TagTypeNotFound { id })?,
            Err(e) => return Err(Error::other(e)),
        };

        let slug = slug.unwrap_or(&tag_type.slug);
        let name = name.unwrap_or(&tag_type.name);
        let kana = kana.unwrap_or(&tag_type.kana);

        let (sql, values) = Query::update()
            .table(PostgresTagType::Table)
            .value(PostgresTagType::Slug, slug)
            .value(PostgresTagType::Name, name)
            .value(PostgresTagType::Kana, kana)
            .and_where(Expr::col(PostgresTagType::Id).eq(PostgresTagTypeId::from(id)))
            .returning(
                Query::returning()
                    .columns([
                        PostgresTagType::Id,
                        PostgresTagType::Slug,
                        PostgresTagType::Name,
                        PostgresTagType::Kana,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let tag_type = match sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => TagType::from(row),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => return Err(ErrorKind::TagTypeSlugDuplicate { slug: slug.to_string() })?,
            Err(e) => return Err(Error::other(e)),
        };

        tx.commit().await.map_err(Error::other)?;
        Ok(tag_type)
    }

    #[tracing::instrument(skip_all)]
    async fn delete_by_id(&self, id: TagTypeId) -> Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresTagType::Table)
            .and_where(Expr::col(PostgresTagType::Id).eq(PostgresTagTypeId::from(id)))
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
