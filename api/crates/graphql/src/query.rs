use std::{marker::PhantomData, sync::Arc};

use async_graphql::{
    connection::{query, Connection},
    Context, Object,
};
use domain::{
    entity::objects::EntryUrlPath,
    repository,
    service::{
        external_services::ExternalServicesServiceInterface,
        media::MediaServiceInterface,
        tags::TagsServiceInterface,
    },
};
use futures::future::try_join_all;
use normalizer::NormalizerInterface;
use uuid::Uuid;

use crate::{
    error::{Error, ErrorKind, Result},
    external_services::ExternalService,
    media::{Medium, MediumCursor},
    objects::{ObjectEntry, ObjectKind},
    replicas::Replica,
    sources::{ExternalMetadata, ExternalMetadataLike, Source},
    tags::{get_tag_depth, Tag, TagCursor, TagTagTypeInput, TagType},
    Order,
};

#[derive(Default)]
pub struct Query<ExternalServicesService, MediaService, TagsService, Normalizer> {
    external_services_service: PhantomData<fn() -> ExternalServicesService>,
    media_service: PhantomData<fn() -> MediaService>,
    tags_service: PhantomData<fn() -> TagsService>,
    normalizer: PhantomData<fn() -> Normalizer>,
}

impl<ExternalServicesService, MediaService, TagsService, Normalizer> Query<ExternalServicesService, MediaService, TagsService, Normalizer> {
    pub fn new() -> Self {
        Self {
            external_services_service: PhantomData,
            media_service: PhantomData,
            tags_service: PhantomData,
            normalizer: PhantomData,
        }
    }
}

#[Object]
impl<ExternalServicesService, MediaService, TagsService, Normalizer> Query<ExternalServicesService, MediaService, TagsService, Normalizer>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
    Normalizer: NormalizerInterface,
{
    /// Fetches all external services.
    #[tracing::instrument(skip_all)]
    async fn all_external_services(&self, ctx: &Context<'_>) -> Result<Vec<ExternalService>> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();

        let external_services = external_services_service.get_external_services().await?;
        Ok(external_services.into_iter().map(Into::into).collect())
    }

    /// Looks up external services by a list of IDs.
    #[tracing::instrument(skip_all)]
    async fn external_services(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The IDs of the ExternalService objects.")]
        ids: Vec<Uuid>,
    ) -> Result<Vec<ExternalService>> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();

        let ids = ids.into_iter().map(Into::into);

        let external_services = external_services_service.get_external_services_by_ids(ids).await?;
        Ok(external_services.into_iter().map(Into::into).collect())
    }

    /// Fetches media optionally filtered by sources or tags, returning up to 100 results.
    #[tracing::instrument(skip_all)]
    async fn all_media(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The IDs of Source objects. Mutually exclusive with `tagIds`.")]
        source_ids: Option<Vec<Uuid>>,
        #[graphql(desc = "The IDs of TagType and Tag objects. Mutually exclusive with `sourceIds`.")]
        tag_ids: Option<Vec<TagTagTypeInput>>,
        #[graphql(default, desc = "The ordering direction of media sorted by `createdAt`.")]
        order: Order,
        #[graphql(desc = "Returns the elements in the list that come after the specified cursor.")]
        after: Option<String>,
        #[graphql(desc = "Returns the elements in the list that come before the specified cursor.")]
        before: Option<String>,
        #[graphql(validator(maximum = 100), desc = "Returns the first _n_ elements from the list.")]
        first: Option<i32>,
        #[graphql(validator(maximum = 100), desc = "Returns the last _n_ elements from the list.")]
        last: Option<i32>,
    ) -> Result<Connection<MediumCursor, Medium>> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let node = match ctx.look_ahead().field("nodes") {
            node if node.exists() => node,
            _ => ctx.look_ahead().field("edges").field("node"),
        };

        let tags = node.field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = node.field("replicas").exists();
        let sources = node.field("sources").exists();

        query(
            after, before, first, last,
            |after, before, first, last| async move {
                let (rev, limit) = match (first, last) {
                    (None, None) => return Err(Error::new(ErrorKind::ArgumentRequired { one_of: vec!["first", "last"] })),
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
                    (Some(_), Some(_)) => return Err(Error::new(ErrorKind::ArgumentsMutuallyExclusive { arguments: vec!["after", "before"] })),
                    (Some(after), _) => Some(MediumCursor::into_inner(after)),
                    (_, Some(before)) => Some(MediumCursor::into_inner(before)),
                    (None, None) => None,
                };

                let media = {
                    let limit = limit as u64 + 1;
                    match (source_ids, tag_ids) {
                        (Some(_), Some(_)) => return Err(Error::new(ErrorKind::ArgumentsMutuallyExclusive { arguments: vec!["sourceIds", "tagIds"] })),
                        (None, None) => {
                            media_service.get_media(tag_depth, replicas, sources, cursor, order, direction, limit).await?
                        },
                        (Some(source_ids), None) => {
                            let source_ids = source_ids.into_iter().map(Into::into);
                            media_service.get_media_by_source_ids(source_ids, tag_depth, replicas, sources, cursor, order, direction, limit).await?
                        },
                        (None, Some(tag_ids)) => {
                            let tag_ids = tag_ids.into_iter().map(Into::into);
                            media_service.get_media_by_tag_ids(tag_ids, tag_depth, replicas, sources, cursor, order, direction, limit).await?
                        },
                    }
                };

                let has_previous = last.is_some() && media.len() > limit;
                let has_next = last.is_none() && media.len() > limit;
                let media = media.into_iter().take(limit);

                let mut connection = Connection::new(has_previous, has_next);
                let result: Result<_> = match rev {
                    true => media.rev().map(|m| Medium::try_from(m).map(Into::into)).collect(),
                    false => media.map(|m| Medium::try_from(m).map(Into::into)).collect(),
                };
                connection.edges = result?;

                Ok(connection)
            },
        ).await.map_err(|e| Error::new(ErrorKind::GraphQLError(e)))
    }

    /// Looks up media by a list of IDs.
    #[tracing::instrument(skip_all)]
    async fn media(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The IDs of Medium objects.")]
        ids: Vec<Uuid>,
    ) -> Result<Vec<Medium>> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let node = ctx.look_ahead();

        let tags = node.field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = node.field("replicas").exists();
        let sources = node.field("sources").exists();

        let ids = ids.into_iter().map(Into::into);

        let media = media_service.get_media_by_ids(ids, tag_depth, replicas, sources).await?;
        media.into_iter().map(TryInto::try_into).collect()
    }

    /// Looks up a replica by its original URL.
    /// ### Errors
    /// * When the replica is not found, it returns a `REPLICA_NOT_FOUND_BY_URL` error.
    #[tracing::instrument(skip_all)]
    async fn replica(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The original URL of media.")]
        original_url: String,
    ) -> Result<Replica> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let replica = media_service.get_replica_by_original_url(&original_url).await?;
        Ok(replica.into())
    }

    /// Looks up sources by partial metadata.
    #[tracing::instrument(skip_all)]
    async fn all_sources_like(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID or URL representing sources.")]
        external_metadata_like: ExternalMetadataLike,
    ) -> Result<Vec<Source>> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();
        let media_service = ctx.data_unchecked::<MediaService>();

        match external_metadata_like {
            ExternalMetadataLike::Id(id) => {
                media_service
                    .get_sources_by_external_metadata_like_id(&id)
                    .await?
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect()
            },
            ExternalMetadataLike::Url(url) => {
                let sources = external_services_service
                    .get_external_services_by_url(&url)
                    .await?
                    .into_iter()
                    .map(|(external_service, external_metadata)| media_service.get_source_by_external_metadata(external_service.id, external_metadata));

                try_join_all(sources)
                    .await?
                    .into_iter()
                    .flatten()
                    .map(TryInto::try_into)
                    .collect()
            },
        }
    }

    /// Looks up sources by a list of IDs.
    #[tracing::instrument(skip_all)]
    async fn sources(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The IDs of Source objects.")]
        ids: Vec<Uuid>,
    ) -> Result<Vec<Source>> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let ids = ids.into_iter().map(Into::into);

        let sources = media_service.get_sources_by_ids(ids).await?;
        sources.into_iter().map(TryInto::try_into).collect()
    }

    /// Looks up a source by the ID of an external service and the external metadata.
    #[tracing::instrument(skip_all)]
    async fn source(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of an ExternalService object.")]
        external_service_id: Uuid,
        #[graphql(desc = "The external metadata of a source.")]
        external_metadata: ExternalMetadata,
    ) -> Result<Option<Source>> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let external_service_id = external_service_id.into();
        let external_metadata = external_metadata.try_into()?;

        let source = media_service.get_source_by_external_metadata(external_service_id, external_metadata).await?;
        source.map(TryInto::try_into).transpose()
    }

    /// Fetches all objects in the storage by their prefix and optionally their kind.
    /// ### Errors
    /// * When the prefix is invalid, it returns an `OBJECT_URL_INVALID` error.
    #[tracing::instrument(skip_all)]
    async fn objects(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The prefix of the object from the root. Must begin with `/`.")]
        prefix: String,
        #[graphql(desc = "The kind of the object.")]
        kind: Option<ObjectKind>,
    ) -> Result<Vec<ObjectEntry>> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let kind = kind.map(Into::into);

        let objects = media_service.get_objects(EntryUrlPath::from(prefix), kind).await?;
        Ok(objects.into_iter().map(Into::into).collect())
    }

    /// Fetches tags.
    #[tracing::instrument(skip_all)]
    async fn all_tags(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = false, desc = "Returns the elements from the root of the hierarchy.")]
        root: bool,
        #[graphql(desc = "Returns the elements in the list that come after the specified cursor.")]
        after: Option<String>,
        #[graphql(desc = "Returns the elements in the list that come before the specified cursor.")]
        before: Option<String>,
        #[graphql(validator(maximum = 100), desc = "Returns the first _n_ elements from the list.")]
        first: Option<i32>,
        #[graphql(validator(maximum = 100), desc = "Returns the last _n_ elements from the list.")]
        last: Option<i32>,
    ) -> Result<Connection<TagCursor, Tag>> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let node = match ctx.look_ahead().field("nodes") {
            node if node.exists() => node,
            _ => ctx.look_ahead().field("edges").field("node"),
        };

        let depth = get_tag_depth(&node);

        query(
            after, before, first, last,
            |after, before, first, last| async move {
                let (rev, limit) = match (first, last) {
                    (None, None) => return Err(Error::new(ErrorKind::ArgumentRequired { one_of: vec!["first", "last"] })),
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
                    (Some(_), Some(_)) => return Err(Error::new(ErrorKind::ArgumentsMutuallyExclusive { arguments: vec!["after", "before"] })),
                    (Some(after), _) => Some(TagCursor::into_inner(after)),
                    (_, Some(before)) => Some(TagCursor::into_inner(before)),
                    (None, None) => None,
                };

                let tags = tags_service.get_tags(
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

                Ok(connection)
            },
        ).await.map_err(|e| Error::new(ErrorKind::GraphQLError(e)))
    }

    /// Looks up tags that contains the given name or alias.
    #[tracing::instrument(skip_all)]
    async fn all_tags_like(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(chars_min_length = 1), desc = "The characters like the name or alias.")]
        name_or_alias_like: String,
    ) -> Result<Vec<Tag>> {
        let tags_service = ctx.data_unchecked::<TagsService>();
        let normalizer = ctx.data_unchecked::<Arc<Normalizer>>();

        let name_or_alias_like = normalizer.normalize(name_or_alias_like);
        let depth = get_tag_depth(&ctx.look_ahead());

        let tags = tags_service.get_tags_by_name_or_alias_like(&name_or_alias_like, depth).await?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    /// Looks up tags by a list of IDs.
    #[tracing::instrument(skip_all)]
    async fn tags(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The IDs of Tag objects.")]
        ids: Vec<Uuid>,
    ) -> Result<Vec<Tag>> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let depth = get_tag_depth(&ctx.look_ahead());
        let ids = ids.into_iter().map(Into::into);

        let tags = tags_service.get_tags_by_ids(ids, depth).await?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    /// Fetches all tag types.
    #[tracing::instrument(skip_all)]
    async fn all_tag_types(&self, ctx: &Context<'_>) -> Result<Vec<TagType>> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let tag_types = tags_service.get_tag_types().await?;
        Ok(tag_types.into_iter().map(Into::into).collect())
    }

    /// Looks up tag types by a list of IDs.
    #[tracing::instrument(skip_all)]
    async fn tag_types(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The IDs of TagType objects.")]
        ids: Vec<Uuid>,
    ) -> Result<Vec<TagType>> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let ids = ids.into_iter().map(Into::into);

        let tag_types = tags_service.get_tag_types_by_ids(ids).await?;
        Ok(tag_types.into_iter().map(Into::into).collect())
    }
}
