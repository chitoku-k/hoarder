use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::entity::tags::TagId;
use sea_query::{ColumnDef, ColumnType, Expr, ExprTrait, ForeignKey, ForeignKeyAction, Index, OnConflict, PgFunc, PostgresQueryBuilder, Query, Table};
use sea_query_sqlx::SqlxBinder;
use sqlx::{Connection, PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};
use uuid::Uuid;

use crate::{
    external_services::PostgresExternalService,
    media::{PostgresMedium, PostgresMediumSource, PostgresMediumTag},
    replicas::{PostgresReplica, PostgresThumbnail},
    sources::PostgresSource,
    tag_types::PostgresTagType,
    tags::{PostgresTag, PostgresTagId, PostgresTagPath},
};

pub(super) struct V1Migration;

impl Migration<Postgres> for V1Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "init"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres>>> {
        vec_box![
            CreateTableOperation,
            CreateIndexOperation,
            InsertRootTagsOperation,
        ]
    }
}

struct CreateTableOperation;

#[async_trait]
impl Operation<Postgres> for CreateTableOperation {
    #[tracing::instrument(skip_all)]
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Table::create()
            .table(PostgresExternalService::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresExternalService::Id).uuid().default(PgFunc::gen_random_uuid()).primary_key())
            .col(ColumnDef::new(PostgresExternalService::Slug).text().not_null().unique_key())
            .col(ColumnDef::new(PostgresExternalService::Name).text().not_null())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresSource::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresSource::Id).uuid().default(PgFunc::gen_random_uuid()).primary_key())
            .col(ColumnDef::new(PostgresSource::ExternalServiceId).uuid().not_null())
            .col(ColumnDef::new(PostgresSource::ExternalMetadata).json_binary().not_null())
            .col(ColumnDef::new(PostgresSource::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(PostgresSource::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresSource::Table, PostgresSource::ExternalServiceId)
                    .to(PostgresExternalService::Table, PostgresExternalService::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .index(
                Index::create()
                    .col(PostgresSource::ExternalServiceId)
                    .col(PostgresSource::ExternalMetadata)
                    .unique(),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresMedium::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresMedium::Id).uuid().default(PgFunc::gen_random_uuid()).primary_key())
            .col(ColumnDef::new(PostgresMedium::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(PostgresMedium::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresMediumSource::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresMediumSource::MediumId).uuid().not_null())
            .col(ColumnDef::new(PostgresMediumSource::SourceId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresMediumSource::Table, PostgresMediumSource::MediumId)
                    .to(PostgresMedium::Table, PostgresMedium::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresMediumSource::Table, PostgresMediumSource::SourceId)
                    .to(PostgresSource::Table, PostgresSource::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .index(
                Index::create()
                    .col(PostgresMediumSource::MediumId)
                    .col(PostgresMediumSource::SourceId)
                    .unique(),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresReplica::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresReplica::Id).uuid().default(PgFunc::gen_random_uuid()).primary_key())
            .col(ColumnDef::new(PostgresReplica::MediumId).uuid().not_null())
            .col(ColumnDef::new(PostgresReplica::DisplayOrder).integer()
                .check(Expr::col(PostgresReplica::DisplayOrder).is_null().or(Expr::col(PostgresReplica::DisplayOrder).gt(0))))
            .col(ColumnDef::new(PostgresReplica::OriginalUrl).text().not_null().unique_key())
            .col(ColumnDef::new(PostgresReplica::MimeType).text().not_null())
            .col(ColumnDef::new(PostgresReplica::Width).integer().not_null())
            .col(ColumnDef::new(PostgresReplica::Height).integer().not_null())
            .col(ColumnDef::new(PostgresReplica::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(PostgresReplica::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresReplica::Table, PostgresReplica::MediumId)
                    .to(PostgresMedium::Table, PostgresMedium::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .index(
                Index::create()
                    .col(PostgresReplica::MediumId)
                    .col(PostgresReplica::DisplayOrder)
                    .unique(),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresThumbnail::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresThumbnail::Id).uuid().default(PgFunc::gen_random_uuid()).primary_key())
            .col(ColumnDef::new(PostgresThumbnail::ReplicaId).uuid().not_null().unique_key())
            .col(ColumnDef::new(PostgresThumbnail::Data).binary())
            .col(ColumnDef::new(PostgresThumbnail::Width).integer().not_null())
            .col(ColumnDef::new(PostgresThumbnail::Height).integer().not_null())
            .col(ColumnDef::new(PostgresThumbnail::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(PostgresThumbnail::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresThumbnail::Table, PostgresThumbnail::ReplicaId)
                    .to(PostgresReplica::Table, PostgresReplica::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresTag::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresTag::Id).uuid().default(PgFunc::gen_random_uuid()).primary_key())
            .col(ColumnDef::new(PostgresTag::Name).text().not_null())
            .col(ColumnDef::new(PostgresTag::Kana).text().not_null())
            .col(ColumnDef::new(PostgresTag::Aliases).array(ColumnType::Text).not_null())
            .col(ColumnDef::new(PostgresTag::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(PostgresTag::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresTagPath::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresTagPath::AncestorId).uuid().not_null())
            .col(ColumnDef::new(PostgresTagPath::DescendantId).uuid().not_null())
            .col(ColumnDef::new(PostgresTagPath::Distance).integer().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresTagPath::Table, PostgresTagPath::AncestorId)
                    .to(PostgresTag::Table, PostgresTag::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresTagPath::Table, PostgresTagPath::DescendantId)
                    .to(PostgresTag::Table, PostgresTag::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .index(
                Index::create()
                    .col(PostgresTagPath::DescendantId)
                    .col(PostgresTagPath::Distance)
                    .unique(),
            )
            .check(
                Expr::col(PostgresTagPath::AncestorId).eq(Uuid::nil()).and(Expr::col(PostgresTagPath::DescendantId).eq(Uuid::nil()))
                    .or(Expr::col(PostgresTagPath::DescendantId).ne(Uuid::nil())),
            )
            .check(
                Expr::col(PostgresTagPath::Distance).gt(0).and(Expr::col(PostgresTagPath::AncestorId).ne(Expr::col(PostgresTagPath::DescendantId)))
                    .or(Expr::col(PostgresTagPath::Distance).eq(0).and(Expr::col(PostgresTagPath::AncestorId).eq(Expr::col(PostgresTagPath::DescendantId)))),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresTagType::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresTagType::Id).uuid().default(PgFunc::gen_random_uuid()).primary_key())
            .col(ColumnDef::new(PostgresTagType::Slug).text().not_null().unique_key())
            .col(ColumnDef::new(PostgresTagType::Name).text().not_null())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::create()
            .table(PostgresMediumTag::Table)
            .if_not_exists()
            .col(ColumnDef::new(PostgresMediumTag::MediumId).uuid().not_null())
            .col(ColumnDef::new(PostgresMediumTag::TagId).uuid().not_null())
            .col(ColumnDef::new(PostgresMediumTag::TagTypeId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresMediumTag::Table, PostgresMediumTag::MediumId)
                    .to(PostgresMedium::Table, PostgresMedium::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresMediumTag::Table, PostgresMediumTag::TagId)
                    .to(PostgresTag::Table, PostgresTag::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(PostgresMediumTag::Table, PostgresMediumTag::TagTypeId)
                    .to(PostgresTagType::Table, PostgresTagType::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .index(
                Index::create()
                    .col(PostgresMediumTag::MediumId)
                    .col(PostgresMediumTag::TagId)
                    .col(PostgresMediumTag::TagTypeId)
                    .unique(),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Table::drop()
            .table(PostgresMediumTag::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresTagType::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresTagPath::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresTag::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresThumbnail::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresReplica::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresMediumSource::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresMedium::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresSource::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::drop()
            .table(PostgresExternalService::Table)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}

struct CreateIndexOperation;

#[async_trait]
impl Operation<Postgres> for CreateIndexOperation {
    #[tracing::instrument(skip_all)]
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Index::create()
            .name("media_created_at_id_idx")
            .if_not_exists()
            .table(PostgresMedium::Table)
            .col(PostgresMedium::CreatedAt)
            .col(PostgresMedium::Id)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Index::create()
            .name("replicas_medium_id_idx")
            .if_not_exists()
            .table(PostgresReplica::Table)
            .col(PostgresReplica::MediumId)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Index::create()
            .name("tags_aliases_idx")
            .if_not_exists()
            .table(PostgresTag::Table)
            .col(PostgresTag::Aliases)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Index::create()
            .name("tags_kana_id_idx")
            .if_not_exists()
            .table(PostgresTag::Table)
            .col(PostgresTag::Kana)
            .col(PostgresTag::Id)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Index::create()
            .name("tag_paths_distance_ancestor_id_descendant_id_idx")
            .if_not_exists()
            .table(PostgresTagPath::Table)
            .col(PostgresTagPath::Distance)
            .col(PostgresTagPath::AncestorId)
            .col(PostgresTagPath::DescendantId)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let sql = Index::drop()
            .name("media_created_at_id_idx")
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Index::drop()
            .name("replicas_medium_id_idx")
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Index::drop()
            .name("tags_aliases_idx")
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Index::drop()
            .name("tags_kana_id_idx")
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Index::drop()
            .name("tag_paths_distance_ancestor_id_descendant_id_idx")
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}

struct InsertRootTagsOperation;

#[async_trait]
impl Operation<Postgres> for InsertRootTagsOperation {
    #[tracing::instrument(skip_all)]
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let (sql, values) = Query::insert()
            .into_table(PostgresTag::Table)
            .columns([
                PostgresTag::Id,
                PostgresTag::Name,
                PostgresTag::Kana,
                PostgresTag::Aliases,
                PostgresTag::CreatedAt,
                PostgresTag::UpdatedAt,
            ])
            .values_panic([
                Expr::value(PostgresTagId::from(TagId::root())),
                Expr::value("root"),
                Expr::value("root"),
                Expr::value(Vec::<String>::new()),
                Expr::value(DateTime::<Utc>::default()),
                Expr::value(DateTime::<Utc>::default()),
            ])
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *connection).await?;

        let (sql, values) = Query::insert()
            .into_table(PostgresTagPath::Table)
            .columns([
                PostgresTagPath::AncestorId,
                PostgresTagPath::DescendantId,
                PostgresTagPath::Distance,
            ])
            .values_panic([
                Expr::value(PostgresTagId::from(TagId::root())),
                Expr::value(PostgresTagId::from(TagId::root())),
                Expr::value(0),
            ])
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        let mut tx = connection.begin().await?;

        let (sql, values) = Query::delete()
            .from_table(PostgresTagPath::Table)
            .and_where(
                Expr::col(PostgresTagPath::AncestorId).eq(PostgresTagId::from(TagId::root()))
                    .and(Expr::col(PostgresTagPath::DescendantId).eq(PostgresTagId::from(TagId::root()))),
            )
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *tx).await?;

        let (sql, values) = Query::delete()
            .from_table(PostgresTag::Table)
            .and_where(Expr::col(PostgresTag::Id).eq(PostgresTagId::from(TagId::root())))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(())
    }
}
