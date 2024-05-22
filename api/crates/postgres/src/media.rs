use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use derive_more::{Constructor, From, Into};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService},
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{Tag, TagDepth, TagId},
    },
    error::{Error, ErrorKind, Result},
    repository::{self, media::MediaRepository, DeleteResult},
};
use futures::{future::ready, TryStreamExt};
use indexmap::IndexSet;
use sea_query::{Alias, BinOper, Expr, Iden, JoinType, Keyword, LockType, OnConflict, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Json, FromRow, PgConnection, PgPool};

use crate::{
    expr::{array::ArrayExpr, distinct::Distinct},
    external_services::{PostgresExternalService, PostgresExternalServiceId},
    replicas::{PostgresMediumReplica, PostgresReplica, PostgresReplicaId, PostgresReplicaThumbnail, PostgresReplicaThumbnailRow, PostgresThumbnail},
    sea_query_uuid_value,
    sources::{PostgresExternalServiceMetadata, PostgresExternalServiceMetadataExtra, PostgresExternalServiceMetadataFull, PostgresSource, PostgresSourceExternalService, PostgresSourceId},
    tag_types::{PostgresTagTagType, PostgresTagType, PostgresTagTypeId},
    tags::{self, PostgresTag, PostgresTagId, PostgresTagPath},
};

#[derive(Clone, Constructor)]
pub struct PostgresMediaRepository {
    pool: PgPool,
}

#[derive(Clone, Debug, From, Into)]
pub(crate) struct PostgresMediumId(MediumId);

#[derive(Debug, FromRow)]
struct PostgresMediumRow {
    id: PostgresMediumId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct PostgresMediumReplicaRow {
    id: PostgresMediumId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    replica_id: PostgresReplicaId,
}

#[derive(Debug, FromRow)]
struct PostgresMediumSourceExternalServiceRow {
    medium_id: PostgresMediumId,
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
}

#[derive(Debug, FromRow)]
struct PostgresMediumTagTypeRow {
    medium_id: PostgresMediumId,
    tag_id: PostgresTagId,
    tag_type_id: PostgresTagTypeId,
    tag_type_slug: String,
    tag_type_name: String,
}

#[derive(Iden)]
pub(crate) enum PostgresMedium {
    #[iden = "media"]
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub(crate) enum PostgresMediumSource {
    #[iden = "media_sources"]
    Table,
    MediumId,
    SourceId,
}

#[derive(Iden)]
pub(crate) enum PostgresMediumTag {
    #[iden = "media_tags"]
    Table,
    MediumId,
    TagId,
    TagTypeId,
}

sea_query_uuid_value!(PostgresMediumId, MediumId);

impl From<PostgresMediumRow> for Medium {
    fn from(row: PostgresMediumRow) -> Self {
        Self {
            id: row.id.into(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            ..Default::default()
        }
    }
}

impl From<PostgresMediumReplicaRow> for (Medium, ReplicaId) {
    fn from(row: PostgresMediumReplicaRow) -> Self {
        (
            Medium {
                id: row.id.into(),
                created_at: row.created_at,
                updated_at: row.updated_at,
                ..Default::default()
            },
            row.replica_id.into(),
        )
    }
}

impl TryFrom<PostgresMediumSourceExternalServiceRow> for (MediumId, Source) {
    type Error = Error;

    fn try_from(row: PostgresMediumSourceExternalServiceRow) -> Result<Self> {
        let external_metadata = PostgresExternalServiceMetadataFull(row.source_external_metadata.0, row.source_external_metadata_extra.0);
        let external_metadata = ExternalMetadata::try_from(external_metadata)?;

        Ok((
            row.medium_id.into(),
            Source {
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
            },
        ))
    }
}

impl From<PostgresMediumTagTypeRow> for (MediumId, TagId, TagType) {
    fn from(row: PostgresMediumTagTypeRow) -> Self {
        (
            row.medium_id.into(),
            row.tag_id.into(),
            TagType {
                id: row.tag_type_id.into(),
                slug: row.tag_type_slug,
                name: row.tag_type_name,
            },
        )
    }
}

impl FromIterator<PostgresMediumTagTypeRow> for HashMap<MediumId, BTreeMap<TagType, Vec<TagId>>> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = PostgresMediumTagTypeRow>,
    {
        iter
            .into_iter()
            .fold(HashMap::new(), |mut media_ids, row| {
                let (medium_id, tag_id, tag_type) = row.into();

                let tag_types = media_ids.entry(medium_id).or_default();
                let tag_ids = tag_types.entry(tag_type).or_default();
                tag_ids.push(tag_id);

                media_ids
            })
    }
}

async fn fetch_tags<T>(conn: &mut PgConnection, ids: T, tag_depth: TagDepth) -> Result<HashMap<MediumId, BTreeMap<TagType, Vec<Tag>>>>
where
    T: IntoIterator<Item = MediumId>,
{
    let ancestors = Alias::new("ancestors");
    let display_order = Alias::new("display_order");

    let (sql, values) = Query::select()
        .expr(Expr::col((PostgresMediumTag::Table, PostgresMediumTag::MediumId)))
        .expr(Expr::col((PostgresMediumTag::Table, PostgresMediumTag::TagId)))
        .expr_as(Expr::col((PostgresTagType::Table, PostgresTagType::Id)), PostgresTagTagType::TagTypeId)
        .expr_as(Expr::col((PostgresTagType::Table, PostgresTagType::Slug)), PostgresTagTagType::TagTypeSlug)
        .expr_as(Expr::col((PostgresTagType::Table, PostgresTagType::Name)), PostgresTagTagType::TagTypeName)
        .from(PostgresMediumTag::Table)
        .join(
            JoinType::InnerJoin,
            PostgresTagType::Table,
            Expr::col((PostgresTagType::Table, PostgresTagType::Id))
                .equals((PostgresMediumTag::Table, PostgresMediumTag::TagTypeId)),
        )
        .join_subquery(
            JoinType::InnerJoin,
            Query::select()
                .expr(Expr::col((PostgresTag::Table, PostgresTag::Id)))
                .expr_as(
                    ArrayExpr::agg(Expr::cust_with_exprs("$1 ORDER BY $2 DESC", [
                        Expr::col((ancestors.clone(), PostgresTag::Kana)).into(),
                        Expr::col((PostgresTagPath::Table, PostgresTagPath::Distance)).into(),
                    ])),
                    display_order.clone(),
                )
                .from(PostgresTag::Table)
                .join(
                    JoinType::InnerJoin,
                    PostgresTagPath::Table,
                    Expr::col((PostgresTagPath::Table, PostgresTagPath::DescendantId))
                        .equals((PostgresTag::Table, PostgresTag::Id)),
                )
                .join_as(
                    JoinType::InnerJoin,
                    PostgresTag::Table,
                    ancestors.clone(),
                    Expr::col((PostgresTagPath::Table, PostgresTagPath::AncestorId))
                        .equals((ancestors.clone(), PostgresTag::Id)),
                )
                .and_where(Expr::col(PostgresTagPath::AncestorId).ne(PostgresTagId::from(TagId::root())))
                .group_by_col((PostgresTag::Table, PostgresTag::Id))
                .order_by(display_order.clone(), Order::Asc)
                .take(),
            ancestors.clone(),
            Expr::col((ancestors, PostgresTag::Id))
                .equals((PostgresMediumTag::Table, PostgresMediumTag::TagId)),
        )
        .and_where(Expr::col((PostgresMediumTag::Table, PostgresMediumTag::MediumId)).is_in(ids.into_iter().map(PostgresMediumId::from)))
        .order_by(PostgresMediumTag::MediumId, Order::Asc)
        .order_by(PostgresMediumTag::TagTypeId, Order::Asc)
        .order_by(display_order, Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let rows: Vec<_> = sqlx::query_as_with::<_, PostgresMediumTagTypeRow, _>(&sql, values)
        .fetch(&mut *conn)
        .try_collect()
        .await
        .map_err(Error::other)?;

    let tag_ids = rows.iter().map(|r| TagId::from(r.tag_id.clone()));
    let tag_relatives: HashMap<_, _> = tags::fetch_tag_relatives(&mut *conn, tag_ids, tag_depth, false)
        .await?
        .into_iter()
        .map(|tag| (tag.id, tag))
        .collect();

    let media_tag_types: HashMap<_, BTreeMap<_, Vec<_>>> = rows.into_iter().collect();
    let tags = media_tag_types
        .into_iter()
        .map(|(medium_id, tag_types)| (
            medium_id,
            tag_types
                .into_iter()
                .map(|(tag_type, tag_ids)| (
                    tag_type,
                    tag_ids
                        .into_iter()
                        .filter_map(|tag_id| tag_relatives.get(&tag_id).cloned())
                        .collect(),
                ))
                .collect(),
        ))
        .collect();

    Ok(tags)
}

async fn fetch_replicas<T>(conn: &mut PgConnection, ids: T) -> Result<HashMap<MediumId, Vec<Replica>>>
where
    T: IntoIterator<Item = MediumId>,
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
        .and_where(Expr::col(PostgresReplica::MediumId).is_in(ids.into_iter().map(PostgresMediumId::from)))
        .order_by(PostgresReplica::MediumId, Order::Asc)
        .order_by(PostgresReplica::DisplayOrder, Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let replicas = sqlx::query_as_with::<_, PostgresReplicaThumbnailRow, _>(&sql, values)
        .fetch(&mut *conn)
        .map_ok(Into::into)
        .try_fold(HashMap::<_, Vec<_>>::new(), |mut replicas, (medium_id, replica)| async move {
            replicas
                .entry(medium_id)
                .or_default()
                .push(replica);
            Ok(replicas)
        })
        .await
        .map_err(Error::other)?;

    Ok(replicas)
}

async fn fetch_sources<T>(conn: &mut PgConnection, ids: T) -> Result<HashMap<MediumId, Vec<Source>>>
where
    T: IntoIterator<Item = MediumId>,
{
    let (sql, values) = Query::select()
        .column(PostgresMediumSource::MediumId)
        .column(PostgresMediumSource::SourceId)
        .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadata)), PostgresSourceExternalService::SourceExternalMetadata)
        .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadataExtra)), PostgresSourceExternalService::SourceExternalMetadataExtra)
        .expr_as(Expr::col((PostgresSource::Table, PostgresSource::CreatedAt)), PostgresSourceExternalService::SourceCreatedAt)
        .expr_as(Expr::col((PostgresSource::Table, PostgresSource::UpdatedAt)), PostgresSourceExternalService::SourceUpdatedAt)
        .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Id)), PostgresSourceExternalService::ExternalServiceId)
        .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Slug)), PostgresSourceExternalService::ExternalServiceSlug)
        .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Kind)), PostgresSourceExternalService::ExternalServiceKind)
        .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Name)), PostgresSourceExternalService::ExternalServiceName)
        .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::BaseUrl)), PostgresSourceExternalService::ExternalServiceBaseUrl)
        .from(PostgresMediumSource::Table)
        .join(
            JoinType::InnerJoin,
            PostgresSource::Table,
            Expr::col((PostgresSource::Table, PostgresSource::Id))
                .equals((PostgresMediumSource::Table, PostgresMediumSource::SourceId))
        )
        .join(
            JoinType::InnerJoin,
            PostgresExternalService::Table,
            Expr::col((PostgresExternalService::Table, PostgresExternalService::Id))
                .equals((PostgresSource::Table, PostgresSource::ExternalServiceId))
        )
        .and_where(Expr::col((PostgresMediumSource::Table, PostgresMediumSource::MediumId)).is_in(ids.into_iter().map(PostgresMediumId::from)))
        .order_by((PostgresMediumSource::Table, PostgresMediumSource::MediumId), Order::Asc)
        .order_by((PostgresMediumSource::Table, PostgresMediumSource::SourceId), Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let sources = sqlx::query_as_with::<_, PostgresMediumSourceExternalServiceRow, _>(&sql, values)
        .fetch(conn)
        .map_err(Error::other)
        .and_then(|row| ready(row.try_into()))
        .try_fold(HashMap::<_, Vec<_>>::new(), |mut sources, (medium_id, source)| async move {
            sources
                .entry(medium_id)
                .or_default()
                .push(source);
            Ok(sources)
        })
        .await?;

    Ok(sources)
}

async fn eager_load(conn: &mut PgConnection, media: &mut [Medium], tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> Result<()> {
    if let Some(tag_depth) = tag_depth {
        let media_ids = media.iter().map(|m| m.id);
        let mut media_tags = fetch_tags(conn, media_ids, tag_depth).await?;

        for medium in media.iter_mut() {
            medium.tags = media_tags.remove(&medium.id).unwrap_or_default();
        }
    }

    if replicas {
        let media_ids = media.iter().map(|m| m.id);
        let mut media_replicas = fetch_replicas(conn, media_ids).await?;

        for medium in media.iter_mut() {
            medium.replicas = media_replicas.remove(&medium.id).unwrap_or_default();
        }
    }

    if sources {
        let media_ids = media.iter().map(|m| m.id);
        let mut media_sources = fetch_sources(conn, media_ids).await?;

        for medium in media.iter_mut() {
            medium.sources = media_sources.remove(&medium.id).unwrap_or_default();
        }
    }

    Ok(())
}

impl MediaRepository for PostgresMediaRepository {
    async fn create<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> Result<Medium>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
    {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let mut query = Query::insert();
        if let Some(created_at) = created_at {
            query.columns([PostgresMedium::CreatedAt])
                .values([created_at.into()])
                .map_err(Error::other)?;
        }

        let (sql, values) = query
            .into_table(PostgresMedium::Table)
            .or_default_values()
            .returning(
                Query::returning()
                    .columns([
                        PostgresMedium::Id,
                        PostgresMedium::CreatedAt,
                        PostgresMedium::UpdatedAt,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let medium: Medium = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch_one(&mut *tx)
            .await
            .map_err(Error::other)?
            .into();

        let query = {
            let mut source_ids = source_ids.into_iter().peekable();
            if source_ids.peek().is_some() {
                let mut query = Query::insert();
                query
                    .into_table(PostgresMediumSource::Table)
                    .columns([
                        PostgresMediumSource::MediumId,
                        PostgresMediumSource::SourceId,
                    ]);

                for source_id in source_ids {
                    query
                        .values([
                            PostgresMediumId::from(medium.id).into(),
                            PostgresSourceId::from(source_id).into(),
                        ])
                        .map_err(Error::other)?;
                }

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            match sqlx::query_with(&sql, values).execute(&mut *tx).await {
                Ok(_) => (),
                Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::MediumSourceNotFound { id: medium.id })?,
                Err(e) => return Err(Error::other(e)),
            }
        }

        let query = {
            let mut tag_tag_type_ids = tag_tag_type_ids.into_iter().peekable();
            if tag_tag_type_ids.peek().is_some() {
                let mut query = Query::insert();
                query
                    .into_table(PostgresMediumTag::Table)
                    .columns([
                        PostgresMediumTag::MediumId,
                        PostgresMediumTag::TagId,
                        PostgresMediumTag::TagTypeId,
                    ]);

                for (tag_id, tag_type_id) in tag_tag_type_ids {
                    query
                        .values([
                            PostgresMediumId::from(medium.id).into(),
                            PostgresTagId::from(tag_id).into(),
                            PostgresTagTypeId::from(tag_type_id).into(),
                        ])
                        .map_err(Error::other)?;
                }

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            match sqlx::query_with(&sql, values).execute(&mut *tx).await {
                Ok(_) => (),
                Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::MediumTagNotFound { id: medium.id })?,
                Err(e) => return Err(Error::other(e)),
            }
        }

        let mut media = [medium];
        eager_load(&mut tx, &mut media, tag_depth, false, sources).await?;

        tx.commit().await.map_err(Error::other)?;

        let [medium] = media;
        Ok(medium)
    }

    async fn fetch_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> Result<Vec<Medium>>
    where
        T: IntoIterator<Item = MediumId> + Send + Sync + 'static,
    {
        let mut conn = self.pool.acquire().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresMedium::Id,
                PostgresMedium::CreatedAt,
                PostgresMedium::UpdatedAt,
            ])
            .from(PostgresMedium::Table)
            .and_where(Expr::col(PostgresMedium::Id).is_in(ids.into_iter().map(PostgresMediumId::from)))
            .order_by(PostgresMedium::CreatedAt, Order::Asc)
            .build_sqlx(PostgresQueryBuilder);

        let mut media: Vec<_> = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch(&mut *conn)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        eager_load(&mut conn, &mut media, tag_depth, replicas, sources).await?;
        Ok(media)
    }

    async fn fetch_by_source_ids<T>(
        &self,
        source_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: repository::Order,
        direction: repository::Direction,
        limit: u64,
    ) -> Result<Vec<Medium>>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
    {
        let mut conn = self.pool.acquire().await.map_err(Error::other)?;

        let (comparison, order, rev) = match (order, direction) {
            (repository::Order::Ascending, repository::Direction::Forward) => (BinOper::GreaterThan, Order::Asc, false),
            (repository::Order::Ascending, repository::Direction::Backward) => (BinOper::SmallerThan, Order::Desc, true),
            (repository::Order::Descending, repository::Direction::Forward) => (BinOper::SmallerThan, Order::Desc, false),
            (repository::Order::Descending, repository::Direction::Backward) => (BinOper::GreaterThan, Order::Asc, true),
        };

        let (sql, values) = Query::select()
            .columns([
                PostgresMedium::Id,
                PostgresMedium::CreatedAt,
                PostgresMedium::UpdatedAt,
            ])
            .from(PostgresMedium::Table)
            .join(
                JoinType::InnerJoin,
                PostgresMediumSource::Table,
                Expr::col((PostgresMediumSource::Table, PostgresMediumSource::MediumId))
                    .equals((PostgresMedium::Table, PostgresMedium::Id)),
            )
            .and_where_option(
                cursor.map(|(created_at, medium_id)| {
                    Expr::tuple([
                        Expr::col(PostgresMedium::CreatedAt).into(),
                        Expr::col(PostgresMedium::Id).into(),
                    ]).binary(comparison, Expr::tuple([
                        Expr::value(created_at),
                        Expr::value(PostgresMediumId::from(medium_id)),
                    ]))
                })
            )
            .and_where(Expr::col(PostgresMediumSource::SourceId).is_in(source_ids.into_iter().map(PostgresSourceId::from)))
            .group_by_col(PostgresMedium::Id)
            .order_by((PostgresMedium::Table, PostgresMedium::CreatedAt), order.clone())
            .order_by((PostgresMedium::Table, PostgresMedium::Id), order)
            .limit(limit)
            .build_sqlx(PostgresQueryBuilder);

        let mut media: Vec<_> = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch(&mut *conn)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        if rev {
            media.reverse();
        }

        eager_load(&mut conn, &mut media, tag_depth, replicas, sources).await?;
        Ok(media)
    }

    async fn fetch_by_tag_ids<T>(
        &self,
        tag_tag_type_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: repository::Order,
        direction: repository::Direction,
        limit: u64,
    ) -> Result<Vec<Medium>>
    where
        T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
    {
        let tag_tag_type_ids: Vec<_> = tag_tag_type_ids
            .into_iter()
            .map(|(tag_id, tag_type_id)| (*tag_id, *tag_type_id))
            .collect();

        let tag_tag_type_ids_len = tag_tag_type_ids.len() as i32;

        let mut conn = self.pool.acquire().await.map_err(Error::other)?;

        let (comparison, order, rev) = match (order, direction) {
            (repository::Order::Ascending, repository::Direction::Forward) => (BinOper::GreaterThan, Order::Asc, false),
            (repository::Order::Ascending, repository::Direction::Backward) => (BinOper::SmallerThan, Order::Desc, true),
            (repository::Order::Descending, repository::Direction::Forward) => (BinOper::SmallerThan, Order::Desc, false),
            (repository::Order::Descending, repository::Direction::Backward) => (BinOper::GreaterThan, Order::Asc, true),
        };

        let (sql, values) = Query::select()
            .columns([
                PostgresMedium::Id,
                PostgresMedium::CreatedAt,
                PostgresMedium::UpdatedAt,
            ])
            .from(PostgresMedium::Table)
            .join(
                JoinType::InnerJoin,
                PostgresMediumTag::Table,
                Expr::col((PostgresMediumTag::Table, PostgresMediumTag::MediumId))
                    .equals((PostgresMedium::Table, PostgresMedium::Id)),
            )
            .join(
                JoinType::InnerJoin,
                PostgresTagPath::Table,
                Expr::col((PostgresTagPath::Table, PostgresTagPath::DescendantId))
                    .equals((PostgresMediumTag::Table, PostgresMediumTag::TagId)),
            )
            .and_where_option(
                cursor.map(|(created_at, medium_id)| {
                    Expr::tuple([
                        Expr::col(PostgresMedium::CreatedAt).into(),
                        Expr::col(PostgresMedium::Id).into(),
                    ]).binary(comparison, Expr::tuple([
                        Expr::value(created_at),
                        Expr::value(PostgresMediumId::from(medium_id)),
                    ]))
                })
            )
            .and_where(
                Expr::tuple([
                    Expr::col(PostgresTagPath::AncestorId).into(),
                    Expr::col(PostgresMediumTag::TagTypeId).into(),
                ]).in_tuples(tag_tag_type_ids)
            )
            .group_by_col(PostgresMedium::Id)
            .and_having(
                Expr::expr(
                    Distinct::arg(
                        Expr::tuple([
                            Expr::col(PostgresTagPath::AncestorId).into(),
                            Expr::col(PostgresMediumTag::TagTypeId).into(),
                        ]),
                    ),
                ).count().eq(Expr::val(tag_tag_type_ids_len))
            )
            .order_by((PostgresMedium::Table, PostgresMedium::CreatedAt), order.clone())
            .order_by((PostgresMedium::Table, PostgresMedium::Id), order)
            .limit(limit)
            .build_sqlx(PostgresQueryBuilder);

        let mut media: Vec<_> = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch(&mut *conn)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        if rev {
            media.reverse();
        }

        eager_load(&mut conn, &mut media, tag_depth, replicas, sources).await?;
        Ok(media)
    }

    async fn fetch_all(
        &self,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: repository::Order,
        direction: repository::Direction,
        limit: u64,
    ) -> Result<Vec<Medium>> {
        let mut conn = self.pool.acquire().await.map_err(Error::other)?;

        let (comparison, order, rev) = match (order, direction) {
            (repository::Order::Ascending, repository::Direction::Forward) => (BinOper::GreaterThan, Order::Asc, false),
            (repository::Order::Ascending, repository::Direction::Backward) => (BinOper::SmallerThan, Order::Desc, true),
            (repository::Order::Descending, repository::Direction::Forward) => (BinOper::SmallerThan, Order::Desc, false),
            (repository::Order::Descending, repository::Direction::Backward) => (BinOper::GreaterThan, Order::Asc, true),
        };

        let (sql, values) = Query::select()
            .columns([
                PostgresMedium::Id,
                PostgresMedium::CreatedAt,
                PostgresMedium::UpdatedAt,
            ])
            .from(PostgresMedium::Table)
            .and_where_option(
                cursor.map(|(created_at, medium_id)| {
                    Expr::tuple([
                        Expr::col(PostgresMedium::CreatedAt).into(),
                        Expr::col(PostgresMedium::Id).into(),
                    ]).binary(comparison, Expr::tuple([
                        Expr::value(created_at),
                        Expr::value(PostgresMediumId::from(medium_id)),
                    ]))
                })
            )
            .order_by(PostgresMedium::CreatedAt, order.clone())
            .order_by(PostgresMedium::Id, order)
            .limit(limit)
            .build_sqlx(PostgresQueryBuilder);

        let mut media: Vec<_> = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch(&mut *conn)
            .map_ok(Into::into)
            .try_collect()
            .await
            .map_err(Error::other)?;

        if rev {
            media.reverse();
        }

        eager_load(&mut conn, &mut media, tag_depth, replicas, sources).await?;
        Ok(media)
    }

    async fn update_by_id<T, U, V, W, X>(
        &self,
        id: MediumId,
        add_source_ids: T,
        remove_source_ids: U,
        add_tag_tag_type_ids: V,
        remove_tag_tag_type_ids: W,
        replica_orders: X,
        created_at: Option<DateTime<Utc>>,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
    ) -> Result<Medium>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        V: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        W: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        X: IntoIterator<Item = ReplicaId> + Send + Sync + 'static,
    {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .exprs([
                Expr::col((PostgresMedium::Table, PostgresMedium::Id)),
                Expr::col((PostgresMedium::Table, PostgresMedium::CreatedAt)),
                Expr::col((PostgresMedium::Table, PostgresMedium::UpdatedAt)),
            ])
            .expr_as(Expr::col((PostgresReplica::Table, PostgresReplica::Id)), PostgresMediumReplica::ReplicaId)
            .from(PostgresMedium::Table)
            .join(
                JoinType::InnerJoin,
                PostgresReplica::Table,
                Expr::col((PostgresReplica::Table, PostgresReplica::MediumId))
                    .equals((PostgresMedium::Table, PostgresMedium::Id)),
            )
            .and_where(Expr::col((PostgresMedium::Table, PostgresMedium::Id)).eq(PostgresMediumId::from(id)))
            .order_by((PostgresMedium::Table, PostgresMedium::Id), Order::Asc)
            .order_by((PostgresReplica::Table, PostgresReplica::DisplayOrder), Order::Asc)
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let replica_ids: IndexSet<_> = sqlx::query_as_with::<_, PostgresMediumReplicaRow, _>(&sql, values)
            .fetch(&mut *tx)
            .map_ok(<(Medium, ReplicaId)>::from)
            .map_ok(|(_, replica_id)| replica_id)
            .try_collect()
            .await
            .map_err(Error::other)?;

        let replica_orders: IndexSet<_> = replica_orders.into_iter().collect();
        if !replica_orders.is_empty() {
            if replica_orders != replica_ids {
                let expected_replicas = replica_ids.into_iter().collect();
                let actual_replicas = replica_orders.into_iter().collect();
                return Err(ErrorKind::MediumReplicasNotMatch { medium_id: id, expected_replicas, actual_replicas })?;
            }

            let (sql, values) = Query::update()
                .table(PostgresReplica::Table)
                .value(PostgresReplica::DisplayOrder, Keyword::Null)
                .and_where(Expr::col(PostgresReplica::MediumId).eq(PostgresMediumId::from(id)))
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(Error::other)?;

            for (order, replica_id) in replica_orders.into_iter().enumerate() {
                let (sql, values) = Query::update()
                    .table(PostgresReplica::Table)
                    .value(PostgresReplica::DisplayOrder, Expr::val(order as i32 + 1))
                    .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(replica_id)))
                    .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map_err(Error::other)?;
            }
        }

        let query = {
            let mut add_source_ids = add_source_ids.into_iter().peekable();
            if add_source_ids.peek().is_some() {
                let mut query = Query::insert();
                query
                    .into_table(PostgresMediumSource::Table)
                    .columns([PostgresMediumSource::MediumId, PostgresMediumSource::SourceId])
                    .on_conflict(OnConflict::new().do_nothing().to_owned());

                for source_id in add_source_ids {
                    query
                        .values([
                            PostgresMediumId::from(id).into(),
                            PostgresSourceId::from(source_id).into(),
                        ])
                        .map_err(Error::other)?;
                }

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            match sqlx::query_with(&sql, values).execute(&mut *tx).await {
                Ok(_) => (),
                Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::MediumSourceNotFound { id })?,
                Err(e) => return Err(Error::other(e)),
            }
        }

        let query = {
            let mut remove_source_ids = remove_source_ids.into_iter().peekable();
            if remove_source_ids.peek().is_some() {
                let mut query = Query::delete();
                query
                    .from_table(PostgresMediumSource::Table)
                    .and_where(Expr::col(PostgresMediumSource::SourceId).is_in(remove_source_ids.map(PostgresSourceId::from)));

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(Error::other)?;
        }

        let query = {
            let mut add_tag_tag_type_ids = add_tag_tag_type_ids.into_iter().peekable();
            if add_tag_tag_type_ids.peek().is_some() {
                let mut query = Query::insert();
                query
                    .into_table(PostgresMediumTag::Table)
                    .columns([
                        PostgresMediumTag::MediumId,
                        PostgresMediumTag::TagId,
                        PostgresMediumTag::TagTypeId,
                    ])
                    .on_conflict(OnConflict::new().do_nothing().to_owned());

                for (tag_id, tag_type_id) in add_tag_tag_type_ids {
                    query
                        .values([
                            PostgresMediumId::from(id).into(),
                            PostgresTagId::from(tag_id).into(),
                            PostgresTagTypeId::from(tag_type_id).into(),
                        ])
                        .map_err(Error::other)?;
                }

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            match sqlx::query_with(&sql, values).execute(&mut *tx).await {
                Ok(_) => (),
                Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::MediumTagNotFound { id })?,
                Err(e) => return Err(Error::other(e)),
            }
        }

        let query = {
            let mut remove_tag_tag_type_ids = remove_tag_tag_type_ids.into_iter().peekable();
            if remove_tag_tag_type_ids.peek().is_some() {
                let remove_tag_tag_type_ids: Vec<_> = remove_tag_tag_type_ids
                    .map(|(tag_id, tag_type_id)| (*tag_id, *tag_type_id))
                    .collect();

                let mut query = Query::delete();
                query
                    .from_table(PostgresMediumTag::Table)
                    .and_where(Expr::col(PostgresMediumTag::MediumId).eq(PostgresMediumId::from(id)))
                    .and_where(
                        Expr::tuple([
                            Expr::col(PostgresMediumTag::TagId).into(),
                            Expr::col(PostgresMediumTag::TagTypeId).into(),
                        ]).in_tuples(remove_tag_tag_type_ids),
                    );

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(Error::other)?;
        }

        let mut query = Query::update();
        query
            .table(PostgresMedium::Table)
            .value(PostgresMedium::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(PostgresMedium::Id).eq(PostgresMediumId::from(id)))
            .returning(
                Query::returning()
                    .columns([
                        PostgresMedium::Id,
                        PostgresMedium::CreatedAt,
                        PostgresMedium::UpdatedAt,
                    ])
            );

        if let Some(created_at) = created_at {
            query.value(PostgresMedium::CreatedAt, created_at);
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let medium = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch_one(&mut *tx)
            .await
            .map_err(Error::other)?
            .into();

        let mut media = [medium];
        eager_load(&mut tx, &mut media, tag_depth, replicas, sources).await?;

        tx.commit().await.map_err(Error::other)?;

        let [medium] = media;
        Ok(medium)
    }

    async fn delete_by_id(&self, id: MediumId) -> Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresMedium::Table)
            .and_where(Expr::col(PostgresMedium::Id).eq(PostgresMediumId::from(id)))
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
