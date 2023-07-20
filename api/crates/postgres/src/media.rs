use std::collections::{BTreeMap, HashMap};

use async_trait::async_trait;
use chrono::NaiveDateTime;
use derive_more::{Constructor, From, Into};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService},
        media::{Medium, MediumError, MediumId},
        replicas::{Replica, ReplicaId},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{Tag, TagDepth, TagId},
    },
    repository::{self, media::MediaRepository, DeleteResult},
};
use futures::TryStreamExt;
use indexmap::IndexSet;
use sea_query::{Expr, Iden, JoinType, Keyword, LockType, OnConflict, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Json, FromRow, PgConnection, PgPool};
use thiserror::Error;

use crate::{
    expr::distinct::Distinct,
    external_services::{PostgresExternalService, PostgresExternalServiceId, PostgresExternalServiceError},
    replicas::{PostgresMediumReplica, PostgresReplica, PostgresReplicaId, PostgresReplicaRow},
    sea_query_uuid_value,
    sources::{PostgresExternalServiceMetadata, PostgresSource, PostgresSourceId, PostgresSourceExternalService},
    tag_types::{PostgresTagTagType, PostgresTagTypeId, PostgresTagType},
    tags::{self, PostgresTagId, PostgresTagPath},
    OrderDirection,
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
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, FromRow)]
struct PostgresMediumReplicaRow {
    id: PostgresMediumId,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    replica_id: PostgresReplicaId,
}

#[derive(Debug, FromRow)]
struct PostgresMediumSourceExternalServiceRow {
    medium_id: PostgresMediumId,
    source_id: PostgresSourceId,
    source_external_metadata: Json<PostgresExternalServiceMetadata>,
    source_created_at: NaiveDateTime,
    source_updated_at: NaiveDateTime,
    external_service_id: PostgresExternalServiceId,
    external_service_slug: String,
    external_service_name: String,
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
enum PostgresMedium {
    #[iden = "media"]
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PostgresMediumSource {
    #[iden = "media_sources"]
    Table,
    MediumId,
    SourceId,
}

#[derive(Iden)]
enum PostgresMediumTag {
    #[iden = "media_tags"]
    Table,
    MediumId,
    TagId,
    TagTypeId,
}

#[derive(Debug, Error)]
pub(crate) enum PostgresMediumError {
    #[error("replicas do not match the actual entries")]
    ReplicaNotMatch,
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
    type Error = serde_json::Error;

    fn try_from(row: PostgresMediumSourceExternalServiceRow) -> serde_json::Result<Self> {
        let external_metadata = ExternalMetadata::try_from(row.source_external_metadata.0)?;

        Ok((
            row.medium_id.into(),
            Source {
                id: row.source_id.into(),
                external_service: ExternalService {
                    id: row.external_service_id.into(),
                    slug: row.external_service_slug,
                    name: row.external_service_name,
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

async fn fetch_tags<T>(conn: &mut PgConnection, ids: T, tag_depth: TagDepth) -> anyhow::Result<HashMap<MediumId, BTreeMap<TagType, Vec<Tag>>>>
where
    T: IntoIterator<Item = MediumId>,
{
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
        .and_where(Expr::col((PostgresMediumTag::Table, PostgresMediumTag::MediumId)).is_in(ids.into_iter().map(PostgresMediumId::from)))
        .order_by(PostgresMediumTag::MediumId, Order::Asc)
        .order_by(PostgresMediumTag::TagTypeId, Order::Asc)
        .order_by(PostgresMediumTag::TagId, Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let rows: Vec<_> = sqlx::query_as_with::<_, PostgresMediumTagTypeRow, _>(&sql, values)
        .fetch(&mut *conn)
        .try_collect()
        .await?;

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

async fn fetch_replicas<T>(conn: &mut PgConnection, ids: T) -> anyhow::Result<HashMap<MediumId, Vec<Replica>>>
where
    T: IntoIterator<Item = MediumId>,
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
        .and_where(Expr::col(PostgresReplica::MediumId).is_in(ids.into_iter().map(PostgresMediumId::from)))
        .order_by(PostgresReplica::MediumId, Order::Asc)
        .order_by(PostgresReplica::DisplayOrder, Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let mut replicas: HashMap<_, Vec<_>> = HashMap::new();
    let mut stream = sqlx::query_as_with::<_, PostgresReplicaRow, _>(&sql, values).fetch(&mut *conn);

    while let Some((medium_id, replica)) = stream.try_next().await?.map(Into::into) {
        replicas
            .entry(medium_id)
            .or_default()
            .push(replica);
    }

    Ok(replicas)
}

async fn fetch_sources<T>(conn: &mut PgConnection, ids: T) -> anyhow::Result<HashMap<MediumId, Vec<Source>>>
where
    T: IntoIterator<Item = MediumId>,
{
    let (sql, values) = Query::select()
        .column(PostgresMediumSource::MediumId)
        .column(PostgresMediumSource::SourceId)
        .expr_as(Expr::col((PostgresSource::Table, PostgresSource::ExternalMetadata)), PostgresSourceExternalService::SourceExternalMetadata)
        .expr_as(Expr::col((PostgresSource::Table, PostgresSource::CreatedAt)), PostgresSourceExternalService::SourceCreatedAt)
        .expr_as(Expr::col((PostgresSource::Table, PostgresSource::UpdatedAt)), PostgresSourceExternalService::SourceUpdatedAt)
        .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Id)), PostgresSourceExternalService::ExternalServiceId)
        .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Slug)), PostgresSourceExternalService::ExternalServiceSlug)
        .expr_as(Expr::col((PostgresExternalService::Table, PostgresExternalService::Name)), PostgresSourceExternalService::ExternalServiceName)
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

    let mut sources = HashMap::<_, Vec<_>>::new();
    let mut stream = sqlx::query_as_with::<_, PostgresMediumSourceExternalServiceRow, _>(&sql, values).fetch(conn);

    while let Some(row) = stream.try_next().await? {
        let (medium_id, source) = match row.try_into() {
             Ok((medium_id, source)) => (medium_id, source),
             Err(e) => Err(PostgresExternalServiceError::Serialize(e))?,
        };
        sources
            .entry(medium_id)
            .or_default()
            .push(source);
    }

    Ok(sources)
}

async fn eager_load(conn: &mut PgConnection, media: &mut [Medium], tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> anyhow::Result<()> {
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

#[async_trait]
impl MediaRepository for PostgresMediaRepository {
    async fn create<T, U>(&self, source_ids: T, created_at: Option<NaiveDateTime>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> anyhow::Result<Medium>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
    {
        let mut tx = self.pool.begin().await?;

        let mut query = Query::insert();
        if let Some(created_at) = created_at {
            query.columns([PostgresMedium::CreatedAt]).values([created_at.into()])?;
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
            .await?
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
                    query.values([
                        PostgresMediumId::from(medium.id).into(),
                        PostgresSourceId::from(source_id).into(),
                    ])?;
                }

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
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
                    query.values([
                        PostgresMediumId::from(medium.id).into(),
                        PostgresTagId::from(tag_id).into(),
                        PostgresTagTypeId::from(tag_type_id).into(),
                    ])?;
                }

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
        }

        let mut media = [medium];
        eager_load(&mut tx, &mut media, tag_depth, false, sources).await?;

        tx.commit().await?;

        let [medium] = media;
        Ok(medium)
    }

    async fn fetch_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = MediumId> + Send + Sync + 'static,
    {
        let mut conn = self.pool.acquire().await?;

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
            .await?;

        eager_load(&mut conn, &mut media, tag_depth, replicas, sources).await?;
        Ok(media)
    }

    async fn fetch_by_source_ids<T>(
        &self,
        source_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: repository::OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
    {
        let mut conn = self.pool.acquire().await?;
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
                since.map(|(created_at, medium_id)| {
                    Expr::tuple([Expr::col(PostgresMedium::CreatedAt).into(), Expr::col(PostgresMedium::Id).into()])
                        .gt(Expr::tuple([Expr::value(created_at), Expr::value(PostgresMediumId::from(medium_id))]))
                })
            )
            .and_where_option(
                until.map(|(created_at, medium_id)| {
                    Expr::tuple([Expr::col(PostgresMedium::CreatedAt).into(), Expr::col(PostgresMedium::Id).into()])
                        .lt(Expr::tuple([Expr::value(created_at), Expr::value(PostgresMediumId::from(medium_id))]))
                })
            )
            .and_where(Expr::col(PostgresMediumSource::SourceId).is_in(source_ids.into_iter().map(PostgresSourceId::from)))
            .group_by_col(PostgresMedium::Id)
            .order_by((PostgresMedium::Table, PostgresMedium::CreatedAt), OrderDirection::from(order).into())
            .order_by((PostgresMedium::Table, PostgresMedium::Id), OrderDirection::from(order).into())
            .limit(limit)
            .build_sqlx(PostgresQueryBuilder);

        let mut media: Vec<_> = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch(&mut *conn)
            .map_ok(Into::into)
            .try_collect()
            .await?;

        eager_load(&mut conn, &mut media, tag_depth, replicas, sources).await?;
        Ok(media)
    }

    async fn fetch_by_tag_ids<T>(
        &self,
        tag_tag_type_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: repository::OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
    {
        let tag_tag_type_ids: Vec<_> = tag_tag_type_ids
            .into_iter()
            .map(|(tag_id, tag_type_id)| (*tag_id, *tag_type_id))
            .collect();

        let tag_tag_type_ids_len = tag_tag_type_ids.len() as i32;

        let mut conn = self.pool.acquire().await?;
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
                since.map(|(created_at, medium_id)| {
                    Expr::tuple([Expr::col(PostgresMedium::CreatedAt).into(), Expr::col(PostgresMedium::Id).into()])
                        .gt(Expr::tuple([Expr::value(created_at), Expr::value(PostgresMediumId::from(medium_id))]))
                })
            )
            .and_where_option(
                until.map(|(created_at, medium_id)| {
                    Expr::tuple([Expr::col(PostgresMedium::CreatedAt).into(), Expr::col(PostgresMedium::Id).into()])
                        .lt(Expr::tuple([Expr::value(created_at), Expr::value(PostgresMediumId::from(medium_id))]))
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
            .order_by((PostgresMedium::Table, PostgresMedium::CreatedAt), OrderDirection::from(order).into())
            .order_by((PostgresMedium::Table, PostgresMedium::Id), OrderDirection::from(order).into())
            .limit(limit)
            .build_sqlx(PostgresQueryBuilder);

        let mut media: Vec<_> = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch(&mut *conn)
            .map_ok(Into::into)
            .try_collect()
            .await?;

        eager_load(&mut conn, &mut media, tag_depth, replicas, sources).await?;
        Ok(media)
    }

    async fn fetch_all(
        &self,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: repository::OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>> {
        let mut conn = self.pool.acquire().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresMedium::Id,
                PostgresMedium::CreatedAt,
                PostgresMedium::UpdatedAt,
            ])
            .from(PostgresMedium::Table)
            .and_where_option(
                since.map(|(created_at, medium_id)| {
                    Expr::tuple([
                        Expr::col(PostgresMedium::CreatedAt).into(),
                        Expr::col(PostgresMedium::Id).into(),
                    ]).gt(Expr::tuple([
                        Expr::value(created_at),
                        Expr::value(PostgresMediumId::from(medium_id)),
                    ]))
                })
            )
            .and_where_option(
                until.map(|(created_at, medium_id)| {
                    Expr::tuple([
                        Expr::col(PostgresMedium::CreatedAt).into(),
                        Expr::col(PostgresMedium::Id).into(),
                    ]).lt(Expr::tuple([
                        Expr::value(created_at),
                        Expr::value(PostgresMediumId::from(medium_id)),
                    ]))
                })
            )
            .order_by(PostgresMedium::CreatedAt, OrderDirection::from(order).into())
            .order_by(PostgresMedium::Id, OrderDirection::from(order).into())
            .limit(limit)
            .build_sqlx(PostgresQueryBuilder);

        let mut media: Vec<_> = sqlx::query_as_with::<_, PostgresMediumRow, _>(&sql, values)
            .fetch(&mut *conn)
            .map_ok(Into::into)
            .try_collect()
            .await?;

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
        created_at: Option<NaiveDateTime>,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
    ) -> anyhow::Result<Medium>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        V: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        W: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        X: IntoIterator<Item = ReplicaId> + Send + Sync + 'static,
    {
        let mut tx = self.pool.begin().await?;

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
            .await?;

        if replica_ids.is_empty() {
            return Err(MediumError::NotFound(id))?;
        }

        let replica_orders: IndexSet<_> = replica_orders.into_iter().collect();
        if !replica_orders.is_empty() {
            if replica_orders != replica_ids {
                return Err(PostgresMediumError::ReplicaNotMatch)?;
            }

            let (sql, values) = Query::update()
                .table(PostgresReplica::Table)
                .value(PostgresReplica::DisplayOrder, Keyword::Null)
                .and_where(Expr::col(PostgresReplica::MediumId).eq(PostgresMediumId::from(id)))
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut *tx).await?;

            for (order, replica_id) in replica_orders.into_iter().enumerate() {
                let (sql, values) = Query::update()
                    .table(PostgresReplica::Table)
                    .value(PostgresReplica::DisplayOrder, Expr::val(order as i32 + 1))
                    .and_where(Expr::col(PostgresReplica::Id).eq(PostgresReplicaId::from(replica_id)))
                    .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
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
                    query.values([
                        PostgresMediumId::from(id).into(),
                        PostgresSourceId::from(source_id).into(),
                    ])?;
                }

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
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
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
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
                    query.values([
                        PostgresMediumId::from(id).into(),
                        PostgresTagId::from(tag_id).into(),
                        PostgresTagTypeId::from(tag_type_id).into(),
                    ])?;
                }

                Some(query)
            } else {
                None
            }
        };
        if let Some(query) = query {
            let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
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
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
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
            .await?
            .into();

        let mut media = [medium];
        eager_load(&mut tx, &mut media, tag_depth, replicas, sources).await?;

        tx.commit().await?;

        let [medium] = media;
        Ok(medium)
    }

    async fn delete_by_id(&self, id: MediumId) -> anyhow::Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresMedium::Table)
            .and_where(Expr::col(PostgresMedium::Id).eq(PostgresMediumId::from(id)))
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
