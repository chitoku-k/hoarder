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
}

#[derive(Iden)]
pub(crate) enum PostgresTagType {
    #[iden = "tag_types"]
    Table,
    Id,
    Slug,
    Name,
}

#[derive(Iden)]
pub(crate) enum PostgresTagTagType {
    TagTypeId,
    TagTypeSlug,
    TagTypeName,
}

sea_query_uuid_value!(PostgresTagTypeId, TagTypeId);

impl From<PostgresTagTypeRow> for TagType {
    fn from(row: PostgresTagTypeRow) -> Self {
        Self {
            id: row.id.into(),
            slug: row.slug,
            name: row.name,
        }
    }
}

impl TagTypesRepository for PostgresTagTypesRepository {
    async fn create(&self, slug: &str, name: &str) -> Result<TagType> {
        let (sql, values) = Query::insert()
            .into_table(PostgresTagType::Table)
            .columns([PostgresTagType::Slug, PostgresTagType::Name])
            .values([Expr::val(slug).into(), Expr::val(name).into()])
            .map_err(Error::other)?
            .returning(
                Query::returning()
                    .columns([
                        PostgresTagType::Id,
                        PostgresTagType::Slug,
                        PostgresTagType::Name,
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

    async fn fetch_all(&self) -> Result<Vec<TagType>> {
        let (sql, values) = Query::select()
            .columns([
                PostgresTagType::Id,
                PostgresTagType::Slug,
                PostgresTagType::Name,
            ])
            .from(PostgresTagType::Table)
            .order_by(PostgresTagType::Name, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let tag_types = sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        Ok(tag_types)
    }

    async fn update_by_id<'a>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'a str>) -> Result<TagType> {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresTagType::Id,
                PostgresTagType::Slug,
                PostgresTagType::Name,
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

        let (sql, values) = Query::update()
            .table(PostgresTagType::Table)
            .value(PostgresTagType::Slug, slug)
            .value(PostgresTagType::Name, name)
            .and_where(Expr::col(PostgresTagType::Id).eq(PostgresTagTypeId::from(id)))
            .returning(
                Query::returning()
                    .columns([
                        PostgresTagType::Id,
                        PostgresTagType::Slug,
                        PostgresTagType::Name,
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
