use async_graphql::{
    connection::{query, Connection},
    Context, Object,
};
use derive_more::Constructor;
use domain::{
    repository,
    service::{
        external_services::ExternalServicesServiceInterface,
        media::MediaServiceInterface,
        tags::TagsServiceInterface,
    },
};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    external_services::ExternalService,
    media::{Medium, MediumCursor},
    replicas::Replica,
    sources::{ExternalMetadata, Source},
    tags::{get_tag_depth, Tag, TagCursor, TagTagTypeInput, TagType},
    Order,
};

#[derive(Constructor)]
pub struct Query<ExternalServicesService, MediaService, TagsService> {
    external_services_service: ExternalServicesService,
    media_service: MediaService,
    tags_service: TagsService,
}

#[derive(Debug, Error)]
pub(crate) enum QueryError {
    #[error("first or last is required")]
    PaginationRequired,
    #[error("both {0} and {1} cannot be specified")]
    MutuallyExclusive(&'static str, &'static str),
    #[error("invalid cursor")]
    InvalidCursor,
}

type Map<T, U, V> = std::iter::Map<T, fn(U) -> V>;

#[Object]
impl<ExternalServicesService, MediaService, TagsService> Query<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
{
    async fn all_external_services(&self) -> anyhow::Result<Vec<ExternalService>> {
        let external_services = self.external_services_service.get_external_services().await?;
        Ok(external_services.into_iter().map(Into::into).collect())
    }

    async fn external_services(&self, ids: Vec<Uuid>) -> anyhow::Result<Vec<ExternalService>> {
        let ids: Map<_, _, _> = ids.into_iter().map(Into::into);

        let external_services = self.external_services_service.get_external_services_by_ids(ids).await?;
        Ok(external_services.into_iter().map(Into::into).collect())
    }

    async fn all_media(
        &self,
        ctx: &Context<'_>,
        source_ids: Option<Vec<Uuid>>,
        tag_ids: Option<Vec<TagTagTypeInput>>,
        #[graphql(default)]
        order: Order,
        after: Option<String>,
        before: Option<String>,
        #[graphql(validator(maximum = 100))]
        first: Option<i32>,
        #[graphql(validator(maximum = 100))]
        last: Option<i32>,
    ) -> async_graphql::Result<Connection<MediumCursor, Medium>> {
        let node = ctx.look_ahead().field("nodes");
        let node = node.exists()
            .then_some(node)
            .unwrap_or_else(|| ctx.look_ahead().field("edges").field("node"));

        let tags = node.field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = node.field("replicas").exists();
        let sources = node.field("sources").exists();

        query(
            after, before, first, last,
            |after, before, first, last| async move {
                let (rev, limit) = match (first, last) {
                    (None, None) => return Err(QueryError::PaginationRequired)?,
                    (Some(first), _) => (false, first),
                    (_, Some(last)) => (true, last),
                };
                let order = match rev {
                    true => order.rev().into(),
                    false => order.into(),
                };
                let direction = match (&after, &before, first, last) {
                    (Some(_), _, _, Some(_)) | (_, Some(_), Some(_), _) => repository::Direction::Backward,
                    _ => repository::Direction::Forward,
                };
                let cursor = match (after, before) {
                    (Some(_), Some(_)) => return Err(QueryError::MutuallyExclusive("after", "before"))?,
                    (Some(after), _) => Some(MediumCursor::into_inner(after)),
                    (_, Some(before)) => Some(MediumCursor::into_inner(before)),
                    (None, None) => None,
                };

                let media = {
                    let limit = limit as u64 + 1;
                    match (source_ids, tag_ids) {
                        (Some(_), Some(_)) => return Err(QueryError::MutuallyExclusive("sourceIds", "tagIds"))?,
                        (None, None) => {
                            self.media_service.get_media(tag_depth, replicas, sources, cursor, order, direction, limit).await?
                        },
                        (Some(source_ids), None) => {
                            let source_ids: Map<_, _, _> = source_ids.into_iter().map(Into::into);
                            self.media_service.get_media_by_source_ids(source_ids, tag_depth, replicas, sources, cursor, order, direction, limit).await?
                        },
                        (None, Some(tag_ids)) => {
                            let tag_ids: Map<_, _, _> = tag_ids.into_iter().map(Into::into);
                            self.media_service.get_media_by_tag_ids(tag_ids, tag_depth, replicas, sources, cursor, order, direction, limit).await?
                        },
                    }
                };

                let has_previous = last.is_some() && media.len() > limit;
                let has_next = last.is_none() && media.len() > limit;
                let media = media.into_iter().take(limit);

                let mut connection = Connection::new(has_previous, has_next);
                let result: anyhow::Result<_> = match rev {
                    true => media.rev().map(|m| Medium::try_from(m).map(Into::into)).collect(),
                    false => media.map(|m| Medium::try_from(m).map(Into::into)).collect(),
                };
                connection.edges = result?;

                anyhow::Ok(connection)
            },
        ).await
    }

    async fn media(&self, ctx: &Context<'_>, ids: Vec<Uuid>) -> anyhow::Result<Vec<Medium>> {
        let node = ctx.look_ahead();

        let tags = node.field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = node.field("replicas").exists();
        let sources = node.field("sources").exists();

        let ids: Map<_, _, _> = ids.into_iter().map(Into::into);

        let media = self.media_service.get_media_by_ids(ids, tag_depth, replicas, sources).await?;
        media.into_iter().map(TryInto::try_into).collect()
    }

    async fn replica(&self, original_url: String) -> anyhow::Result<Replica> {
        let replica = self.media_service.get_replica_by_original_url(&original_url).await?;
        Ok(replica.into())
    }

    async fn source(&self, external_service_id: Uuid, external_metadata: ExternalMetadata) -> anyhow::Result<Source> {
        let external_service_id = external_service_id.into();
        let external_metadata = external_metadata.try_into()?;

        let source = self.media_service.get_source_by_external_metadata(external_service_id, external_metadata).await?;
        source.try_into()
    }

    async fn all_tags(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = false)]
        root: bool,
        after: Option<String>,
        before: Option<String>,
        #[graphql(validator(maximum = 100))]
        first: Option<i32>,
        #[graphql(validator(maximum = 100))]
        last: Option<i32>,
    ) -> async_graphql::Result<Connection<TagCursor, Tag>> {
        let node = ctx.look_ahead().field("nodes");
        let node = node.exists()
            .then_some(node)
            .unwrap_or_else(|| ctx.look_ahead().field("edges").field("node"));

        let depth = get_tag_depth(&node);

        query(
            after, before, first, last,
            |after, before, first, last| async move {
                let (rev, limit) = match (first, last) {
                    (None, None) => return Err(QueryError::PaginationRequired)?,
                    (Some(first), _) => (false, first),
                    (_, Some(last)) => (true, last),
                };
                let order = match rev {
                    true => repository::Order::Descending,
                    false => repository::Order::Ascending,
                };
                let direction = match (&after, &before, first, last) {
                    (Some(_), _, _, Some(_)) | (_, Some(_), Some(_), _) => repository::Direction::Backward,
                    _ => repository::Direction::Forward,
                };
                let cursor = match (after, before) {
                    (Some(_), Some(_)) => return Err(QueryError::MutuallyExclusive("after", "before"))?,
                    (Some(after), _) => Some(TagCursor::into_inner(after)),
                    (_, Some(before)) => Some(TagCursor::into_inner(before)),
                    (None, None) => None,
                };

                let tags = self.tags_service.get_tags(
                    depth,
                    root,
                    cursor,
                    order,
                    direction,
                    limit as u64 + 1
                ).await?;

                let has_previous = last.is_some() && tags.len() > limit;
                let has_next = last.is_none() && tags.len() > limit;
                let tags = tags.into_iter().take(limit);

                let mut connection = Connection::new(has_previous, has_next);
                connection.edges = match rev {
                    true => tags.rev().map(Tag::from).map(Into::into).collect(),
                    false => tags.map(Tag::from).map(Into::into).collect(),
                };

                anyhow::Ok(connection)
            },
        ).await
    }

    async fn all_tags_like(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(chars_min_length = 1))]
        name_or_alias_like: String,
    ) -> anyhow::Result<Vec<Tag>> {
        let depth = get_tag_depth(&ctx.look_ahead());

        let tags = self.tags_service.get_tags_by_name_or_alias_like(&name_or_alias_like, depth).await?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    async fn tags(&self, ctx: &Context<'_>, ids: Vec<Uuid>) -> anyhow::Result<Vec<Tag>> {
        let depth = get_tag_depth(&ctx.look_ahead());
        let ids: Map<_, _, _> = ids.into_iter().map(Into::into);

        let tags = self.tags_service.get_tags_by_ids(ids, depth).await?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    async fn all_tag_types(&self) -> anyhow::Result<Vec<TagType>> {
        let tag_types = self.tags_service.get_tag_types().await?;
        Ok(tag_types.into_iter().map(Into::into).collect())
    }
}
