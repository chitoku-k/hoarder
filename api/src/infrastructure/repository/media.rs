use std::collections::{BTreeMap, HashMap};

use async_trait::async_trait;
use chrono::NaiveDateTime;
use derive_more::Constructor;
use futures::TryStreamExt;
use indexmap::IndexSet;
use sea_query::{BinOper, Expr, Iden, JoinType, Keyword, LockType, OnConflict, Order, PostgresQueryBuilder, Query, SimpleExpr};
use sqlx::{types::Json, FromRow, PgConnection, PgPool};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            external_services::{ExternalMetadata, ExternalService},
            media::{Medium, MediumError, MediumId},
            replicas::{Replica, ReplicaId},
            sources::{Source, SourceId},
            tag_types::{TagType, TagTypeId},
            tags::{Tag, TagDepth, TagId},
        },
        repository::{media::MediaRepository, DeleteResult, OrderDirection},
    },
    infrastructure::repository::{
        expr::distinct::Distinct,
        external_services::{PostgresExternalService, PostgresExternalServiceError},
        replicas::{PostgresMediumReplica, PostgresReplica, PostgresReplicaRow},
        sea_query_driver_postgres::{bind_query, bind_query_as}, sea_query_uuid_value,
        sources::{PostgresExternalServiceMetadata, PostgresSource, PostgresSourceExternalService},
        tag_types::{PostgresTagTagType, PostgresTagType},
        tags::{self, PostgresTagPath},
    },
};

#[derive(Clone, Constructor)]
pub struct PostgresMediaRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
struct PostgresMediumRow {
    id: Uuid,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, FromRow)]
struct PostgresMediumReplicaRow {
    id: Uuid,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    replica_id: Uuid,
}

#[derive(Debug, FromRow)]
struct PostgresMediumSourceExternalServiceRow {
    medium_id: Uuid,
    source_id: Uuid,
    source_external_metadata: Json<PostgresExternalServiceMetadata>,
    source_created_at: NaiveDateTime,
    source_updated_at: NaiveDateTime,
    external_service_id: Uuid,
    external_service_slug: String,
    external_service_name: String,
}

#[derive(Debug, FromRow)]
struct PostgresMediumTagTypeRow {
    medium_id: Uuid,
    tag_id: Uuid,
    tag_type_id: Uuid,
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
pub enum PostgresMediumError {
    #[error("replicas do not match the actual entries")]
    ReplicaNotMatch,
}

sea_query_uuid_value!(MediumId);

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
                .equals(PostgresMediumTag::Table, PostgresMediumTag::TagTypeId),
        )
        .and_where(Expr::col((PostgresMediumTag::Table, PostgresMediumTag::MediumId)).is_in(ids))
        .order_by(PostgresMediumTag::MediumId, Order::Asc)
        .order_by(PostgresMediumTag::TagTypeId, Order::Asc)
        .order_by(PostgresMediumTag::TagId, Order::Asc)
        .build(PostgresQueryBuilder);

    let rows: Vec<_> = bind_query_as::<PostgresMediumTagTypeRow>(sqlx::query_as(&sql), &values)
        .fetch(&mut *conn)
        .try_collect()
        .await?;

    let tag_ids = rows.iter().map(|r| TagId::from(r.tag_id));
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
        .and_where(Expr::col(PostgresReplica::MediumId).is_in(ids))
        .order_by(PostgresReplica::MediumId, Order::Asc)
        .order_by(PostgresReplica::DisplayOrder, Order::Asc)
        .build(PostgresQueryBuilder);

    let mut replicas: HashMap<_, Vec<_>> = HashMap::new();
    let mut stream = bind_query_as::<PostgresReplicaRow>(sqlx::query_as(&sql), &values).fetch(&mut *conn);

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
                .equals(PostgresMediumSource::Table, PostgresMediumSource::SourceId)
        )
        .join(
            JoinType::InnerJoin,
            PostgresExternalService::Table,
            Expr::col((PostgresExternalService::Table, PostgresExternalService::Id))
                .equals(PostgresSource::Table, PostgresSource::ExternalServiceId)
        )
        .and_where(Expr::col((PostgresMediumSource::Table, PostgresMediumSource::MediumId)).is_in(ids))
        .order_by((PostgresMediumSource::Table, PostgresMediumSource::MediumId), Order::Asc)
        .order_by((PostgresMediumSource::Table, PostgresMediumSource::SourceId), Order::Asc)
        .build(PostgresQueryBuilder);

    let mut sources = HashMap::<_, Vec<_>>::new();
    let mut stream = bind_query_as::<PostgresMediumSourceExternalServiceRow>(sqlx::query_as(&sql), &values).fetch(conn);

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
    async fn create(
        &self,
        source_ids: Vec<SourceId>,
        created_at: Option<NaiveDateTime>,
        tag_tag_type_ids: Vec<(TagId, TagTypeId)>,
        tag_depth: Option<TagDepth>,
        sources: bool,
    ) -> anyhow::Result<Medium> {
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
            .build(PostgresQueryBuilder);

        let medium: Medium = bind_query_as::<PostgresMediumRow>(sqlx::query_as(&sql), &values)
            .fetch_one(&mut tx)
            .await
            .map(Into::into)?;

        if !source_ids.is_empty() {
            let mut query = Query::insert();
            query
                .into_table(PostgresMediumSource::Table)
                .columns([
                    PostgresMediumSource::MediumId,
                    PostgresMediumSource::SourceId,
                ]);

            for source_id in source_ids {
                query.values([
                    medium.id.into(),
                    source_id.into(),
                ])?;
            }

            let (sql, values) = query.build(PostgresQueryBuilder);
            bind_query(sqlx::query(&sql), &values).execute(&mut tx).await?;
        }

        if !tag_tag_type_ids.is_empty() {
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
                    medium.id.into(),
                    tag_id.into(),
                    tag_type_id.into(),
                ])?;
            }

            let (sql, values) = query.build(PostgresQueryBuilder);
            bind_query(sqlx::query(&sql), &values).execute(&mut tx).await?;
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
            .and_where(Expr::col(PostgresMedium::Id).is_in(ids))
            .order_by(PostgresMedium::CreatedAt, Order::Asc)
            .build(PostgresQueryBuilder);

        let mut media: Vec<_> = bind_query_as::<PostgresMediumRow>(sqlx::query_as(&sql), &values)
            .fetch(&mut conn)
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
        order: OrderDirection,
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
                    .equals(PostgresMedium::Table, PostgresMedium::Id),
            )
            .and_where_option(
                since.map(|(created_at, medium_id)| {
                    Expr::tuple([Expr::col(PostgresMedium::CreatedAt).into(), Expr::col(PostgresMedium::Id).into()])
                        .greater_than(Expr::tuple([Expr::value(created_at), Expr::value(medium_id)]))
                })
            )
            .and_where_option(
                until.map(|(created_at, medium_id)| {
                    Expr::tuple([Expr::col(PostgresMedium::CreatedAt).into(), Expr::col(PostgresMedium::Id).into()])
                        .less_than(Expr::tuple([Expr::value(created_at), Expr::value(medium_id)]))
                })
            )
            .and_where(Expr::col(PostgresMediumSource::SourceId).is_in(source_ids))
            .group_by_col(PostgresMedium::Id)
            .order_by((PostgresMedium::Table, PostgresMedium::CreatedAt), order.into())
            .order_by((PostgresMedium::Table, PostgresMedium::Id), order.into())
            .limit(limit)
            .build(PostgresQueryBuilder);

        let mut media: Vec<_> = bind_query_as::<PostgresMediumRow>(sqlx::query_as(&sql), &values)
            .fetch(&mut conn)
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
        order: OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
    {
        let tag_tag_type_ids: Vec<_> = tag_tag_type_ids
            .into_iter()
            .map(|(tag_id, tag_type_id)| SimpleExpr::Values(vec![tag_id.into(), tag_type_id.into()]))
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
                    .equals(PostgresMedium::Table, PostgresMedium::Id),
            )
            .join(
                JoinType::InnerJoin,
                PostgresTagPath::Table,
                Expr::col((PostgresTagPath::Table, PostgresTagPath::DescendantId))
                    .equals(PostgresMediumTag::Table, PostgresMediumTag::TagId),
            )
            .and_where_option(
                since.map(|(created_at, medium_id)| {
                    Expr::tuple([Expr::col(PostgresMedium::CreatedAt).into(), Expr::col(PostgresMedium::Id).into()])
                        .greater_than(Expr::tuple([Expr::value(created_at), Expr::value(medium_id)]))
                })
            )
            .and_where_option(
                until.map(|(created_at, medium_id)| {
                    Expr::tuple([Expr::col(PostgresMedium::CreatedAt).into(), Expr::col(PostgresMedium::Id).into()])
                        .less_than(Expr::tuple([Expr::value(created_at), Expr::value(medium_id)]))
                })
            )
            .and_where(
                Expr::tuple([
                    Expr::col(PostgresTagPath::AncestorId).into(),
                    Expr::col(PostgresMediumTag::TagTypeId).into(),
                ]).binary(
                    BinOper::In,
                    SimpleExpr::Tuple(tag_tag_type_ids),
                ),
            )
            .group_by_col(PostgresMedium::Id)
            .and_having(
                Distinct::arg(
                    Expr::tuple([
                        Expr::col(PostgresTagPath::AncestorId).into(),
                        Expr::col(PostgresMediumTag::TagTypeId).into(),
                    ]),
                ).count().equals(Expr::val(tag_tag_type_ids_len))
            )
            .order_by((PostgresMedium::Table, PostgresMedium::CreatedAt), order.into())
            .order_by((PostgresMedium::Table, PostgresMedium::Id), order.into())
            .limit(limit)
            .build(PostgresQueryBuilder);

        let mut media: Vec<_> = bind_query_as::<PostgresMediumRow>(sqlx::query_as(&sql), &values)
            .fetch(&mut conn)
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
        order: OrderDirection,
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
                    ]).greater_than(Expr::tuple([
                        Expr::value(created_at),
                        Expr::value(medium_id),
                    ]))
                })
            )
            .and_where_option(
                until.map(|(created_at, medium_id)| {
                    Expr::tuple([
                        Expr::col(PostgresMedium::CreatedAt).into(),
                        Expr::col(PostgresMedium::Id).into(),
                    ]).less_than(Expr::tuple([
                        Expr::value(created_at),
                        Expr::value(medium_id),
                    ]))
                })
            )
            .order_by(PostgresMedium::CreatedAt, order.into())
            .order_by(PostgresMedium::Id, order.into())
            .limit(limit)
            .build(PostgresQueryBuilder);

        let mut media: Vec<_> = bind_query_as::<PostgresMediumRow>(sqlx::query_as(&sql), &values)
            .fetch(&mut conn)
            .map_ok(Into::into)
            .try_collect()
            .await?;

        eager_load(&mut conn, &mut media, tag_depth, replicas, sources).await?;
        Ok(media)
    }

    async fn update_by_id<T>(
        &self,
        id: MediumId,
        add_source_ids: Vec<SourceId>,
        remove_source_ids: Vec<SourceId>,
        add_tag_tag_type_ids: Vec<(TagId, TagTypeId)>,
        remove_tag_tag_type_ids: Vec<(TagId, TagTypeId)>,
        replica_orders: T,
        created_at: Option<NaiveDateTime>,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
    ) -> anyhow::Result<Medium>
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static,
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
                    .equals(PostgresMedium::Table, PostgresMedium::Id),
            )
            .and_where(Expr::col((PostgresMedium::Table, PostgresMedium::Id)).eq(id))
            .order_by((PostgresMedium::Table, PostgresMedium::Id), Order::Asc)
            .order_by((PostgresReplica::Table, PostgresReplica::DisplayOrder), Order::Asc)
            .lock(LockType::Update)
            .build(PostgresQueryBuilder);

        let replica_ids: IndexSet<_> = bind_query_as::<PostgresMediumReplicaRow>(sqlx::query_as(&sql), &values)
            .fetch(&mut tx)
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
                .col_expr(PostgresReplica::DisplayOrder, SimpleExpr::Keyword(Keyword::Null))
                .and_where(Expr::col(PostgresReplica::MediumId).eq(id))
                .build(PostgresQueryBuilder);

            bind_query(sqlx::query(&sql), &values).execute(&mut tx).await?;

            for (order, replica_id) in replica_orders.into_iter().enumerate() {
                let (sql, values) = Query::update()
                    .table(PostgresReplica::Table)
                    .col_expr(PostgresReplica::DisplayOrder, Expr::val(order as i32 + 1).into())
                    .and_where(Expr::col(PostgresReplica::Id).eq(replica_id))
                    .build(PostgresQueryBuilder);

                bind_query(sqlx::query(&sql), &values).execute(&mut tx).await?;
            }
        }

        if !add_source_ids.is_empty() {
            let mut query = Query::insert();
            query
                .into_table(PostgresMediumSource::Table)
                .columns([PostgresMediumSource::MediumId, PostgresMediumSource::SourceId])
                .on_conflict(OnConflict::new().do_nothing().to_owned());

            for source_id in add_source_ids {
                query.values([id.into(), source_id.into()])?;
            }

            let (sql, values) = query.build(PostgresQueryBuilder);
            bind_query(sqlx::query(&sql), &values).execute(&mut tx).await?;
        }

        if !remove_source_ids.is_empty() {
            let (sql, values) = Query::delete()
                .from_table(PostgresMediumSource::Table)
                .and_where(Expr::col(PostgresMediumSource::SourceId).is_in(remove_source_ids))
                .build(PostgresQueryBuilder);

            bind_query(sqlx::query(&sql), &values).execute(&mut tx).await?;
        }

        if !add_tag_tag_type_ids.is_empty() {
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
                    id.into(),
                    tag_id.into(),
                    tag_type_id.into(),
                ])?;
            }

            let (sql, values) = query.build(PostgresQueryBuilder);
            bind_query(sqlx::query(&sql), &values).execute(&mut tx).await?;
        }

        if !remove_tag_tag_type_ids.is_empty() {
            let remove_tag_tag_type_ids: Vec<_> = remove_tag_tag_type_ids
                .into_iter()
                .map(|(tag_id, tag_type_id)| SimpleExpr::Values(vec![tag_id.into(), tag_type_id.into()]))
                .collect();

            let (sql, values) = Query::delete()
                .from_table(PostgresMediumTag::Table)
                .and_where(Expr::col(PostgresMediumTag::MediumId).eq(id))
                .and_where(
                    Expr::tuple([
                        Expr::col(PostgresMediumTag::TagId).into(),
                        Expr::col(PostgresMediumTag::TagTypeId).into(),
                    ]).binary(
                        BinOper::In,
                        SimpleExpr::Tuple(remove_tag_tag_type_ids),
                    ),
                )
                .build(PostgresQueryBuilder);
            bind_query(sqlx::query(&sql), &values).execute(&mut tx).await?;
        }

        let mut query = Query::update();
        query
            .table(PostgresMedium::Table)
            .col_expr(PostgresMedium::UpdatedAt, Expr::cust("CURRENT_TIMESTAMP"))
            .and_where(Expr::col(PostgresMedium::Id).eq(id))
            .returning(
                Query::returning()
                    .columns([
                        PostgresMedium::Id,
                        PostgresMedium::CreatedAt,
                        PostgresMedium::UpdatedAt,
                    ])
            );

        if let Some(created_at) = created_at {
            query.value(PostgresMedium::CreatedAt, created_at.into());
        }

        let (sql, values) = query.build(PostgresQueryBuilder);
        let medium = bind_query_as::<PostgresMediumRow>(sqlx::query_as(&sql), &values)
            .fetch_one(&mut tx)
            .await
            .map(Into::into)?;

        let mut media = [medium];
        eager_load(&mut tx, &mut media, tag_depth, replicas, sources).await?;

        tx.commit().await?;

        let [medium] = media;
        Ok(medium)
    }

    async fn delete_by_id(&self, id: MediumId) -> anyhow::Result<DeleteResult> {
        let (sql, values) = Query::delete()
            .from_table(PostgresMedium::Table)
            .and_where(Expr::col(PostgresMedium::Id).eq(id))
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
    use std::collections::{BTreeMap, BTreeSet};

    use chrono::NaiveDate;
    use compiled_uuid::uuid;
    use pretty_assertions::assert_eq;
    use sqlx::Row;
    use test_context::test_context;

    use crate::{
        domain::entity::{external_services::ExternalServiceId, tags::AliasSet},
        infrastructure::repository::tests::DatabaseContext,
    };

    use super::*;

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.create(
            Vec::new(),
            None,
            Vec::new(),
            None,
            false,
        ).await.unwrap();

        let actual_id = *actual.id;
        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, Vec::new());

        let actual = sqlx::query(r#"SELECT "id" FROM "media" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<Uuid, &str>("id"), actual_id);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_with_created_at_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.create(
            Vec::new(),
            Some(NaiveDate::from_ymd(2022, 1, 1).and_hms(5, 6, 7)),
            Vec::new(),
            None,
            false,
        ).await.unwrap();

        let actual_id = *actual.id;
        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, Vec::new());

        let actual = sqlx::query(r#"SELECT "id", "created_at" FROM "media" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<Uuid, &str>("id"), actual_id);
        assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd(2022, 1, 1).and_hms(5, 6, 7));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_with_sources_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.create(
            vec![
                SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
            ],
            None,
            Vec::new(),
            None,
            true,
        ).await.unwrap();

        let actual_id = *actual.id;
        assert_eq!(actual.sources, vec![
            Source {
                id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
                external_metadata: ExternalMetadata::Twitter { id: 111111111111 },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 15),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 17),
            },
            Source {
                id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, Vec::new());

        let actual = sqlx::query(r#"SELECT "id" FROM "media" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<Uuid, &str>("id"), actual_id);

        let actual: Vec<_> = sqlx::query(r#"SELECT "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(actual_id)
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db"));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_with_tags_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.create(
            Vec::new(),
            None,
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
                (
                    TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Some(TagDepth::new(2, 2)),
            true,
        ).await.unwrap();

        let actual_id = *actual.id;
        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, {
            let mut tags = BTreeMap::new();
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                    slug: "work".to_string(),
                    name: "作品".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                name: "七森中☆ごらく部".to_string(),
                                kana: "ななもりちゅうごらくぶ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                        name: "赤座あかり".to_string(),
                                        kana: "あかざあかり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                        name: "歳納京子".to_string(),
                                        kana: "としのうきょうこ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                        name: "船見結衣".to_string(),
                                        kana: "ふなみゆい".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                        name: "吉川ちなつ".to_string(),
                                        kana: "よしかわちなつ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                ],
            );
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                        name: "歳納京子".to_string(),
                        kana: "としのうきょうこ".to_string(),
                        aliases: AliasSet::default(),
                        parent: Some(Box::new(Tag {
                            id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                            name: "七森中☆ごらく部".to_string(),
                            kana: "ななもりちゅうごらくぶ".to_string(),
                            aliases: AliasSet::default(),
                            parent: Some(Box::new(Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                            updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Tag {
                        id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                        name: "船見結衣".to_string(),
                        kana: "ふなみゆい".to_string(),
                        aliases: AliasSet::default(),
                        parent: Some(Box::new(Tag {
                            id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                            name: "七森中☆ごらく部".to_string(),
                            kana: "ななもりちゅうごらくぶ".to_string(),
                            aliases: AliasSet::default(),
                            parent: Some(Box::new(Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                            updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
            );
            tags
        });
        assert_eq!(actual.replicas, Vec::new());

        let actual = sqlx::query(r#"SELECT "id" FROM "media" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<Uuid, &str>("id"), actual_id);

        let actual: Vec<_> = sqlx::query(r#"SELECT "tag_type_id", "tag_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(actual_id)
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));

        assert_eq!(actual[2].get::<Uuid, &str>("tag_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[2].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_with_sources_tags_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.create(
            vec![
                SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
            ],
            None,
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
                (
                    TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Some(TagDepth::new(2, 2)),
            true,
        ).await.unwrap();

        let actual_id = *actual.id;
        assert_eq!(actual.sources, vec![
            Source {
                id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
                external_metadata: ExternalMetadata::Twitter { id: 111111111111 },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 15),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 17),
            },
            Source {
                id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
        assert_eq!(actual.tags, {
            let mut tags = BTreeMap::new();
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                    slug: "work".to_string(),
                    name: "作品".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                name: "七森中☆ごらく部".to_string(),
                                kana: "ななもりちゅうごらくぶ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                        name: "赤座あかり".to_string(),
                                        kana: "あかざあかり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                        name: "歳納京子".to_string(),
                                        kana: "としのうきょうこ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                        name: "船見結衣".to_string(),
                                        kana: "ふなみゆい".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                        name: "吉川ちなつ".to_string(),
                                        kana: "よしかわちなつ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                ],
            );
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                        name: "歳納京子".to_string(),
                        kana: "としのうきょうこ".to_string(),
                        aliases: AliasSet::default(),
                        parent: Some(Box::new(Tag {
                            id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                            name: "七森中☆ごらく部".to_string(),
                            kana: "ななもりちゅうごらくぶ".to_string(),
                            aliases: AliasSet::default(),
                            parent: Some(Box::new(Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                            updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Tag {
                        id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                        name: "船見結衣".to_string(),
                        kana: "ふなみゆい".to_string(),
                        aliases: AliasSet::default(),
                        parent: Some(Box::new(Tag {
                            id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                            name: "七森中☆ごらく部".to_string(),
                            kana: "ななもりちゅうごらくぶ".to_string(),
                            aliases: AliasSet::default(),
                            parent: Some(Box::new(Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                            updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
            );
            tags
        });
        assert_eq!(actual.replicas, Vec::new());

        let actual = sqlx::query(r#"SELECT "id" FROM "media" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<Uuid, &str>("id"), actual_id);

        let actual: Vec<_> = sqlx::query(r#"SELECT "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(actual_id)
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "tag_type_id", "tag_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(actual_id)
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));

        assert_eq!(actual[2].get::<Uuid, &str>("tag_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[2].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_ids_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_ids(
            vec![
                MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            ],
            None,
            false,
            false,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_ids_with_tags_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_ids(
            vec![
                MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_ids_with_replicas_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_ids(
            vec![
                MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            ],
            None,
            true,
            false,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                        display_order: Some(1),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                        display_order: Some(2),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                        display_order: Some(3),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_ids_with_sources_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_ids(
            vec![
                MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            ],
            None,
            false,
            true,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 12),
                    },
                    Source {
                        id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_ids_with_tags_replicas_sources_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_ids(
            vec![
                MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            ],
            Some(TagDepth::new(2, 2)),
            true,
            true,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 12),
                    },
                    Source {
                        id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                        display_order: Some(1),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                        display_order: Some(2),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                        display_order: Some(3),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_tags_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_replicas_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            true,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                        display_order: Some(1),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                        display_order: Some(2),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                        display_order: Some(3),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_sources_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            true,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 12),
                    },
                    Source {
                        id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_tags_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                name: "フランドール・スカーレット".to_string(),
                                kana: "フランドール・スカーレット".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_replicas_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            true,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("fc874edd-6920-477d-a070-3c28203a070f")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/fc874edd-6920-477d-a070-3c28203a070f.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 12),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                        display_order: Some(1),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                        display_order: Some(2),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                        display_order: Some(3),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_sources_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            true,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 12),
                    },
                    Source {
                        id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_tags_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                name: "フランドール・スカーレット".to_string(),
                                kana: "フランドール・スカーレット".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_replicas_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            true,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                        display_order: Some(1),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                        display_order: Some(2),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                        display_order: Some(3),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("fc874edd-6920-477d-a070-3c28203a070f")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/fc874edd-6920-477d-a070-3c28203a070f.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 12),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_sources_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            true,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 12),
                    },
                    Source {
                        id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_tags_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                name: "フランドール・スカーレット".to_string(),
                                kana: "フランドール・スカーレット".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_replicas_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            true,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("fc874edd-6920-477d-a070-3c28203a070f")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/fc874edd-6920-477d-a070-3c28203a070f.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 12),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                        display_order: Some(1),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                        display_order: Some(2),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                        display_order: Some(3),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_sources_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            true,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 12),
                    },
                    Source {
                        id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_tags_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_replicas_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            true,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_sources_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            true,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_tags_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_replicas_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            true,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_source_ids_with_sources_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_source_ids(
            vec![
                SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
            ],
            None,
            false,
            true,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_tags_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_replicas_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            true,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_sources_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            true,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Source {
                        id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_tags_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_replicas_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            true,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/9b73469d-55fe-4017-aee8-dd8f8d7d067a.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_sources_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            true,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 111111111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 15),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 17),
                    },
                    Source {
                        id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 16),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Source {
                        id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_tags_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_replicas_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            true,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/9b73469d-55fe-4017-aee8-dd8f8d7d067a.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_sources_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            true,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Source {
                        id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 111111111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 15),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 17),
                    },
                    Source {
                        id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 16),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_tags_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_replicas_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            true,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/9b73469d-55fe-4017-aee8-dd8f8d7d067a.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_sources_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            true,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 111111111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 15),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 17),
                    },
                    Source {
                        id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 16),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_tags_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_replicas_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            true,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_sources_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            true,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Source {
                        id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_tags_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_replicas_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            true,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_tag_ids_with_sources_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            None,
            false,
            true,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Source {
                        id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_tags_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
                            slug: "clothes".to_string(),
                            name: "衣装".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                name: "魔女っ娘ミラクるん".to_string(),
                                kana: "まじょっこミラクるん".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_replicas_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            true,
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_sources_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            true,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Source {
                        id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
            Medium {
                id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_tags_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
            Medium {
                id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                name: "博麗霊夢".to_string(),
                                kana: "はくれいれいむ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                name: "フランドール・スカーレット".to_string(),
                                kana: "フランドール・スカーレット".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_replicas_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            true,
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("69f9463e-9c29-48c9-a104-23341348ffec")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/69f9463e-9c29-48c9-a104-23341348ffec.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 17),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("040d009c-70df-4f55-ae55-df6e5fc57362")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/040d009c-70df-4f55-ae55-df6e5fc57362.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 18),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 11),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
            Medium {
                id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1524e043-a327-43ab-9a87-4e5ffa051cb7")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/1524e043-a327-43ab-9a87-4e5ffa051cb7.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 15),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("38505b5a-2e25-4325-8668-97cc39b57e73")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/38505b5a-2e25-4325-8668-97cc39b57e73.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 5),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_sources_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            true,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
            Medium {
                id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("2a82a031-e27a-443e-9f22-bb190f70633a")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 4444, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Source {
                        id: SourceId::from(uuid!("e607c6f5-af17-4f65-9868-b3e72f692f4d")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 5555555 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 13),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Medium {
                id: MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("725792bf-dbf0-4af1-b639-a147f0b327b2")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 2222, creator_id: "creator_02".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 12),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("da2e3cc8-5b12-45fc-b720-815e74fb8fe6")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 6666666 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_tags_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            Some(TagDepth::new(2, 2)),
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                name: "フランドール・スカーレット".to_string(),
                                kana: "フランドール・スカーレット".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_replicas_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            true,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/9b73469d-55fe-4017-aee8-dd8f8d7d067a.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                        display_order: Some(1),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                        display_order: Some(2),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                        display_order: Some(3),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("fc874edd-6920-477d-a070-3c28203a070f")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/fc874edd-6920-477d-a070-3c28203a070f.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 12),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_sources_and_since_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            true,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 111111111111 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 15),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 17),
                    },
                    Source {
                        id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 16),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
            },
            Medium {
                id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 12),
                    },
                    Source {
                        id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Medium {
                id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
            Medium {
                id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_tags_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            Some(TagDepth::new(2, 2)),
            false,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
            Medium {
                id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                        name: "古明地こいし".to_string(),
                                        kana: "こめいじこいし".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                        name: "博麗霊夢".to_string(),
                                        kana: "はくれいれいむ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                        name: "フランドール・スカーレット".to_string(),
                                        kana: "フランドール・スカーレット".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                        name: "鈴仙・優曇華院・イナバ".to_string(),
                                        kana: "れいせん・うどんげいん・いなば".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                name: "博麗霊夢".to_string(),
                                kana: "はくれいれいむ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                    name: "東方Project".to_string(),
                                    kana: "とうほうProject".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_replicas_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            true,
            false,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("69f9463e-9c29-48c9-a104-23341348ffec")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/69f9463e-9c29-48c9-a104-23341348ffec.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 17),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("040d009c-70df-4f55-ae55-df6e5fc57362")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/040d009c-70df-4f55-ae55-df6e5fc57362.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 18),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 11),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
            Medium {
                id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("1524e043-a327-43ab-9a87-4e5ffa051cb7")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/1524e043-a327-43ab-9a87-4e5ffa051cb7.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 15),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_sources_and_since_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            true,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
            Medium {
                id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("2a82a031-e27a-443e-9f22-bb190f70633a")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 4444, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                    Source {
                        id: SourceId::from(uuid!("e607c6f5-af17-4f65-9868-b3e72f692f4d")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 5555555 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 13),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_tags_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_replicas_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            true,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_sources_and_until_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            true,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Source {
                        id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_tags_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            Some(TagDepth::new(2, 2)),
            false,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: vec![
                                    Tag {
                                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                        name: "七森中☆ごらく部".to_string(),
                                        kana: "ななもりちゅうごらくぶ".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: vec![
                                            Tag {
                                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                                name: "船見結衣".to_string(),
                                                kana: "ふなみゆい".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                            },
                                            Tag {
                                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                                name: "吉川ちなつ".to_string(),
                                                kana: "よしかわちなつ".to_string(),
                                                aliases: AliasSet::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                    },
                                    Tag {
                                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                        name: "魔女っ娘ミラクるん".to_string(),
                                        kana: "まじょっこミラクるん".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                                    },
                                ],
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: Some(Box::new(Tag {
                                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                                    name: "七森中☆ごらく部".to_string(),
                                    kana: "ななもりちゅうごらくぶ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: AliasSet::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                                    updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_replicas_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            true,
            false,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                        display_order: Some(1),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                        display_order: Some(2),
                        has_thumbnail: false,
                        original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                        mime_type: "image/jpeg".to_string(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                    },
                ],
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_sources_and_until_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            None,
            false,
            true,
            None,
            Some((NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 9),
                    },
                    Source {
                        id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                            slug: "skeb".to_string(),
                            name: "Skeb".to_string(),
                        },
                        external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 6),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Medium {
                id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                    },
                    Source {
                        id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 14),
                        updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 15),
                    },
                ],
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 5),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Vec::new(),
            None,
            None,
            false,
            false,
        ).await.unwrap();

        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "tag_id", "tag_type_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order" FROM "replicas" WHERE "medium_id" = $1 ORDER BY "display_order""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
        assert_eq!(actual[0].get::<Option<i32>, &str>("display_order"), Some(1));

        assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
        assert_eq!(actual[1].get::<Option<i32>, &str>("display_order"), Some(2));

        assert_eq!(actual[2].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
        assert_eq!(actual[2].get::<Option<i32>, &str>("display_order"), Some(3));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_with_tags_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Vec::new(),
            None,
            Some(TagDepth::new(2, 2)),
            false,
            false,
        ).await.unwrap();

        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, {
            let mut tags = BTreeMap::new();
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                    slug: "work".to_string(),
                    name: "作品".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                        name: "東方Project".to_string(),
                        kana: "とうほうProject".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                            Tag {
                                id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                name: "博麗霊夢".to_string(),
                                kana: "はくれいれいむ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                            },
                            Tag {
                                id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                name: "フランドール・スカーレット".to_string(),
                                kana: "フランドール・スカーレット".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                            Tag {
                                id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                name: "鈴仙・優曇華院・イナバ".to_string(),
                                kana: "れいせん・うどんげいん・いなば".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
            );
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                        name: "フランドール・スカーレット".to_string(),
                        kana: "フランドール・スカーレット".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                        parent: Some(Box::new(Tag {
                            id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                            name: "東方Project".to_string(),
                            kana: "とうほうProject".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                            updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
            );
            tags
        });
        assert_eq!(actual.replicas, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "tag_id", "tag_type_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order" FROM "replicas" WHERE "medium_id" = $1 ORDER BY "display_order""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
        assert_eq!(actual[0].get::<Option<i32>, &str>("display_order"), Some(1));

        assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
        assert_eq!(actual[1].get::<Option<i32>, &str>("display_order"), Some(2));

        assert_eq!(actual[2].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
        assert_eq!(actual[2].get::<Option<i32>, &str>("display_order"), Some(3));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_with_replicas_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Vec::new(),
            None,
            None,
            true,
            false,
        ).await.unwrap();

        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, vec![
            Replica {
                id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                display_order: Some(1),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Replica {
                id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                display_order: Some(2),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Replica {
                id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                display_order: Some(3),
                has_thumbnail: false,
                original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "tag_id", "tag_type_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order" FROM "replicas" WHERE "medium_id" = $1 ORDER BY "display_order""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
        assert_eq!(actual[0].get::<Option<i32>, &str>("display_order"), Some(1));

        assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
        assert_eq!(actual[1].get::<Option<i32>, &str>("display_order"), Some(2));

        assert_eq!(actual[2].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
        assert_eq!(actual[2].get::<Option<i32>, &str>("display_order"), Some(3));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_with_sources_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            Vec::new(),
            None,
            None,
            false,
            true,
        ).await.unwrap();

        assert_eq!(actual.sources, vec![
            Source {
                id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
                external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Source {
                id: SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                    slug: "skeb".to_string(),
                    name: "Skeb".to_string(),
                },
                external_metadata: ExternalMetadata::Skeb { id: 1111, creator_id: "creator_02".to_string() },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 13),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
        ]);
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 7));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "tag_id", "tag_type_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order" FROM "replicas" WHERE "medium_id" = $1 ORDER BY "display_order""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
        assert_eq!(actual[0].get::<Option<i32>, &str>("display_order"), Some(1));

        assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
        assert_eq!(actual[1].get::<Option<i32>, &str>("display_order"), Some(2));

        assert_eq!(actual[2].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
        assert_eq!(actual[2].get::<Option<i32>, &str>("display_order"), Some(3));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_reorder_replicas_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ],
            Some(NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8)),
            None,
            false,
            false,
        ).await.unwrap();

        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7));

        let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "tag_id", "tag_type_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order" FROM "replicas" WHERE "medium_id" = $1 ORDER BY "display_order""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
        assert_eq!(actual[0].get::<Option<i32>, &str>("display_order"), Some(1));

        assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
        assert_eq!(actual[1].get::<Option<i32>, &str>("display_order"), Some(2));

        assert_eq!(actual[2].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
        assert_eq!(actual[2].get::<Option<i32>, &str>("display_order"), Some(3));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_reorder_replicas_with_tags_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ],
            Some(NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8)),
            Some(TagDepth::new(2, 2)),
            false,
            false,
        ).await.unwrap();

        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, {
            let mut tags = BTreeMap::new();
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                    slug: "work".to_string(),
                    name: "作品".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                        name: "東方Project".to_string(),
                        kana: "とうほうProject".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                                name: "古明地こいし".to_string(),
                                kana: "こめいじこいし".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
                            },
                            Tag {
                                id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                name: "博麗霊夢".to_string(),
                                kana: "はくれいれいむ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
                            },
                            Tag {
                                id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                name: "フランドール・スカーレット".to_string(),
                                kana: "フランドール・スカーレット".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                            },
                            Tag {
                                id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                name: "鈴仙・優曇華院・イナバ".to_string(),
                                kana: "れいせん・うどんげいん・いなば".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 8),
                            },
                        ],
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
            );
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                        name: "フランドール・スカーレット".to_string(),
                        kana: "フランドール・スカーレット".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                        parent: Some(Box::new(Tag {
                            id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                            name: "東方Project".to_string(),
                            kana: "とうほうProject".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 8),
                            updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 9),
                        updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
                    },
                ],
            );
            tags
        });
        assert_eq!(actual.replicas, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7));

        let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "tag_id", "tag_type_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order" FROM "replicas" WHERE "medium_id" = $1 ORDER BY "display_order""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
        assert_eq!(actual[0].get::<Option<i32>, &str>("display_order"), Some(1));

        assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
        assert_eq!(actual[1].get::<Option<i32>, &str>("display_order"), Some(2));

        assert_eq!(actual[2].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
        assert_eq!(actual[2].get::<Option<i32>, &str>("display_order"), Some(3));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_reorder_replicas_with_replicas_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ],
            Some(NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8)),
            None,
            true,
            false,
        ).await.unwrap();

        assert_eq!(actual.sources, Vec::new());
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, vec![
            Replica {
                id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                display_order: Some(1),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 10),
            },
            Replica {
                id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                display_order: Some(2),
                has_thumbnail: false,
                original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 11),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
            Replica {
                id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                display_order: Some(3),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 10),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7),
            },
        ]);
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7));

        let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "tag_id", "tag_type_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order" FROM "replicas" WHERE "medium_id" = $1 ORDER BY "display_order""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
        assert_eq!(actual[0].get::<Option<i32>, &str>("display_order"), Some(1));

        assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
        assert_eq!(actual[1].get::<Option<i32>, &str>("display_order"), Some(2));

        assert_eq!(actual[2].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
        assert_eq!(actual[2].get::<Option<i32>, &str>("display_order"), Some(3));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_reorder_replicas_with_sources_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ],
            Some(NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8)),
            None,
            false,
            true,
        ).await.unwrap();

        assert_eq!(actual.sources, vec![
            Source {
                id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
                external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 16),
                updated_at: NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 6),
            },
            Source {
                id: SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                    slug: "skeb".to_string(),
                    name: "Skeb".to_string(),
                },
                external_metadata: ExternalMetadata::Skeb { id: 1111, creator_id: "creator_02".to_string() },
                created_at: NaiveDate::from_ymd(2022, 1, 2).and_hms(3, 4, 13),
                updated_at: NaiveDate::from_ymd(2022, 3, 4).and_hms(5, 6, 11),
            },
        ]);
        assert_eq!(actual.tags, BTreeMap::new());
        assert_eq!(actual.replicas, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8));
        assert_ne!(actual.updated_at, NaiveDate::from_ymd(2022, 2, 3).and_hms(4, 5, 7));

        let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "source_id" FROM "media_sources" WHERE "medium_id" = $1 ORDER BY "source_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("source_id"), uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128"));
        assert_eq!(actual[1].get::<Uuid, &str>("source_id"), uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "medium_id", "tag_id", "tag_type_id" FROM "media_tags" WHERE "medium_id" = $1 ORDER BY "tag_type_id", "tag_id""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("tag_type_id"), uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"));
        assert_eq!(actual[0].get::<Uuid, &str>("tag_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));

        assert_eq!(actual[1].get::<Uuid, &str>("tag_type_id"), uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38"));
        assert_eq!(actual[1].get::<Uuid, &str>("tag_id"), uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d"));

        let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order" FROM "replicas" WHERE "medium_id" = $1 ORDER BY "display_order""#)
            .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
        assert_eq!(actual[0].get::<Option<i32>, &str>("display_order"), Some(1));

        assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
        assert_eq!(actual[1].get::<Option<i32>, &str>("display_order"), Some(2));

        assert_eq!(actual[2].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
        assert_eq!(actual[2].get::<Option<i32>, &str>("display_order"), Some(3));
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_reorder_too_few_replicas_fails(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ],
            Some(NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8)),
            None,
            false,
            false,
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_reorder_too_many_replicas_fails(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
            ],
            Some(NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8)),
            None,
            false,
            false,
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_reorder_replicas_mismatch_fails(ctx: &DatabaseContext) {
        let repository = PostgresMediaRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            vec![
                SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
                SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            ],
            vec![
                SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            ],
            vec![
                (
                    TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                ),
                (
                    TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                (
                    TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                    TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                ),
            ],
            vec![
                ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
            ],
            Some(NaiveDate::from_ymd(2022, 4, 5).and_hms(6, 7, 8)),
            None,
            false,
            false,
        ).await;

        assert!(actual.is_err());
    }
}
