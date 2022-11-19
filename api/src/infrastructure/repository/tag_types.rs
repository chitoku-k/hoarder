use anyhow::Context;
use async_trait::async_trait;
use derive_more::Constructor;
use futures::TryStreamExt;
use sea_query::{Expr, Iden, LockType, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    domain::{
        entity::tag_types::{TagType, TagTypeError, TagTypeId},
        repository::{tag_types::TagTypesRepository, DeleteResult},
    },
    infrastructure::repository::sea_query_uuid_value,
};

#[derive(Clone, Constructor)]
pub struct PostgresTagTypesRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
struct PostgresTagTypeRow {
    id: Uuid,
    slug: String,
    name: String,
}

#[derive(Iden)]
pub enum PostgresTagType {
    #[iden = "tag_types"]
    Table,
    Id,
    Slug,
    Name,
}

#[derive(Iden)]
pub enum PostgresTagTagType {
    TagTypeId,
    TagTypeSlug,
    TagTypeName,
}

sea_query_uuid_value!(TagTypeId);

impl From<PostgresTagTypeRow> for TagType {
    fn from(row: PostgresTagTypeRow) -> Self {
        Self {
            id: row.id.into(),
            slug: row.slug,
            name: row.name,
        }
    }
}

#[async_trait]
impl TagTypesRepository for PostgresTagTypesRepository {
    async fn create(&self, slug: &str, name: &str) -> anyhow::Result<TagType> {
        let (sql, values) = Query::insert()
            .into_table(PostgresTagType::Table)
            .columns([PostgresTagType::Slug, PostgresTagType::Name])
            .values([Expr::val(slug).into(), Expr::val(name).into()])?
            .returning(
                Query::returning()
                    .columns([
                        PostgresTagType::Id,
                        PostgresTagType::Slug,
                        PostgresTagType::Name,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let tag_type = sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?
            .into();

        Ok(tag_type)
    }

    async fn fetch_all(&self) -> anyhow::Result<Vec<TagType>> {
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
            .await?;

        Ok(tag_types)
    }

    async fn update_by_id<'a>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'a str>) -> anyhow::Result<TagType> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresTagType::Id,
                PostgresTagType::Slug,
                PostgresTagType::Name,
            ])
            .from(PostgresTagType::Table)
            .and_where(Expr::col(PostgresTagType::Id).eq(id))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let tag_type: TagType = sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values)
            .fetch_optional(&mut tx)
            .await?
            .map(Into::into)
            .context(TagTypeError::NotFound(id))?;

        let slug = slug.unwrap_or(&tag_type.slug);
        let name = name.unwrap_or(&tag_type.name);

        let (sql, values) = Query::update()
            .table(PostgresTagType::Table)
            .value(PostgresTagType::Slug, slug)
            .value(PostgresTagType::Name, name)
            .and_where(Expr::col(PostgresTagType::Id).eq(id))
            .returning(
                Query::returning()
                    .columns([
                        PostgresTagType::Id,
                        PostgresTagType::Slug,
                        PostgresTagType::Name,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let tag_type = sqlx::query_as_with::<_, PostgresTagTypeRow, _>(&sql, values)
            .fetch_one(&mut tx)
            .await?
            .into();

        tx.commit().await?;
        Ok(tag_type)
    }

    async fn delete_by_id(&self, id: TagTypeId) -> anyhow::Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresTagType::Table)
            .and_where(Expr::col(PostgresTagType::Id).eq(id))
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use sqlx::Row;
    use test_context::test_context;
    use uuid::uuid;

    use crate::infrastructure::repository::tests::DatabaseContext;

    use super::*;

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
        let actual = repository.create("foobar", "FooBar").await.unwrap();

        assert_eq!(actual.slug, "foobar".to_string());
        assert_eq!(actual.name, "FooBar".to_string());

        let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "tag_types" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("slug"), "foobar");
        assert_eq!(actual.get::<&str, &str>("name"), "FooBar");
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_fails(ctx: &DatabaseContext) {
        let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
        let actual = repository.create("character", "キャラクター").await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all().await.unwrap();

        assert_eq!(actual, vec![
            TagType {
                id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                slug: "character".to_string(),
                name: "キャラクター".to_string(),
            },
            TagType {
                id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                slug: "work".to_string(),
                name: "作品".to_string(),
            },
            TagType {
                id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                slug: "clothes".to_string(),
                name: "衣装".to_string(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            None,
            None,
        ).await.unwrap();

        assert_eq!(actual.slug, "work".to_string());
        assert_eq!(actual.name, "作品".to_string());

        let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "tag_types" WHERE "id" = $1"#)
            .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("slug"), "work");
        assert_eq!(actual.get::<&str, &str>("name"), "作品");
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_with_slug_and_name_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            Some("works"),
            Some("版権"),
        ).await.unwrap();

        assert_eq!(actual.slug, "works".to_string());
        assert_eq!(actual.name, "版権".to_string());

        let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "tag_types" WHERE "id" = $1"#)
            .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("slug"), "works");
        assert_eq!(actual.get::<&str, &str>("name"), "版権");
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_fails(ctx: &DatabaseContext) {
        let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            TagTypeId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            Some("illustrators"),
            Some("絵師"),
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_succeeds(ctx: &DatabaseContext) {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tag_types" WHERE "id" = $1"#)
            .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);

        let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
        let actual = repository.delete_by_id(TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(1));

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tag_types" WHERE "id" = $1"#)
            .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 0);

        let actual = repository.delete_by_id(TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))).await.unwrap();

        assert_eq!(actual, DeleteResult::NotFound);
    }
}
