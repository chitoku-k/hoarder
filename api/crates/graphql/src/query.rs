use async_graphql::{
    connection::{query, Connection},
    Context, Object,
};
use derive_more::Constructor;
use domain::{
    entity::objects::EntryPath,
    repository,
    service::{
        external_services::ExternalServicesServiceInterface,
        media::MediaServiceInterface,
        tags::TagsServiceInterface,
    },
};
use uuid::Uuid;

use crate::{
    error::{Error, ErrorKind, Result},
    external_services::ExternalService,
    media::{Medium, MediumCursor},
    objects::{ObjectEntry, ObjectKind},
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

type Map<T, U, V> = std::iter::Map<T, fn(U) -> V>;

#[Object]
impl<ExternalServicesService, MediaService, TagsService> Query<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
{
    async fn all_external_services(&self) -> Result<Vec<ExternalService>> {
        let external_services = self.external_services_service.get_external_services().await?;
        Ok(external_services.into_iter().map(Into::into).collect())
    }

    async fn external_services(&self, ids: Vec<Uuid>) -> Result<Vec<ExternalService>> {
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
    ) -> Result<Connection<MediumCursor, Medium>> {
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
                let result: Result<_> = match rev {
                    true => media.rev().map(|m| Medium::try_from(m).map(Into::into).map_err(Error::new)).collect(),
                    false => media.map(|m| Medium::try_from(m).map(Into::into).map_err(Error::new)).collect(),
                };
                connection.edges = result?;

                Ok(connection)
            },
        ).await.map_err(|e| Error::new(ErrorKind::GraphQLError(e)))
    }

    async fn media(&self, ctx: &Context<'_>, ids: Vec<Uuid>) -> Result<Vec<Medium>> {
        let node = ctx.look_ahead();

        let tags = node.field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = node.field("replicas").exists();
        let sources = node.field("sources").exists();

        let ids: Map<_, _, _> = ids.into_iter().map(Into::into);

        let media = self.media_service.get_media_by_ids(ids, tag_depth, replicas, sources).await?;
        media.into_iter().map(|m| m.try_into().map_err(Error::new)).collect()
    }

    async fn replica(&self, original_url: String) -> Result<Replica> {
        let replica = self.media_service.get_replica_by_original_url(&original_url).await?;
        Ok(replica.into())
    }

    async fn source(&self, external_service_id: Uuid, external_metadata: ExternalMetadata) -> Result<Option<Source>> {
        let external_service_id = external_service_id.into();
        let external_metadata = external_metadata.try_into().map_err(Error::new)?;

        let source = self.media_service.get_source_by_external_metadata(external_service_id, external_metadata).await?;
        source.map(TryInto::try_into).transpose().map_err(Error::new)
    }

    async fn objects(&self, prefix: String, kind: Option<ObjectKind>) -> Result<Vec<ObjectEntry>> {
        let kind = kind.map(Into::into);

        let objects = self.media_service.get_objects(&EntryPath::from(prefix), kind).await?;
        Ok(objects.into_iter().map(Into::into).collect())
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
    ) -> Result<Connection<TagCursor, Tag>> {
        let node = ctx.look_ahead().field("nodes");
        let node = node.exists()
            .then_some(node)
            .unwrap_or_else(|| ctx.look_ahead().field("edges").field("node"));

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

                Ok(connection)
            },
        ).await.map_err(|e| Error::new(ErrorKind::GraphQLError(e)))
    }

    async fn all_tags_like(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(chars_min_length = 1))]
        name_or_alias_like: String,
    ) -> Result<Vec<Tag>> {
        let depth = get_tag_depth(&ctx.look_ahead());

        let tags = self.tags_service.get_tags_by_name_or_alias_like(&name_or_alias_like, depth).await?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    async fn tags(&self, ctx: &Context<'_>, ids: Vec<Uuid>) -> Result<Vec<Tag>> {
        let depth = get_tag_depth(&ctx.look_ahead());
        let ids: Map<_, _, _> = ids.into_iter().map(Into::into);

        let tags = self.tags_service.get_tags_by_ids(ids, depth).await?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    async fn all_tag_types(&self) -> Result<Vec<TagType>> {
        let tag_types = self.tags_service.get_tag_types().await?;
        Ok(tag_types.into_iter().map(Into::into).collect())
    }
}
