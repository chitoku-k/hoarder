use async_graphql::{
    connection::{query, Connection},
    Context, Object,
};
use derive_more::Constructor;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    application::graphql::{
        external_services::ExternalService,
        media::{Medium, MediumCursor},
        replicas::Replica,
        sources::{ExternalMetadata, Source},
        tags::{get_tag_depth, Tag, TagCursor, TagTagTypeInput, TagType},
        OrderDirection,
    },
    domain::{
        repository,
        service::{
            external_services::ExternalServicesServiceInterface,
            media::MediaServiceInterface,
            tags::TagsServiceInterface,
        },
    },
};

#[derive(Constructor)]
pub struct Query<ExternalServicesService, MediaService, TagsService> {
    external_services_service: ExternalServicesService,
    media_service: MediaService,
    tags_service: TagsService,
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("first or last is required")]
    PaginationRequired,
    #[error("both {0} and {1} cannot be specified")]
    MutuallyExclusive(&'static str, &'static str),
    #[error("invalid cursor")]
    InvalidCursor,
}

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
        let ids = ids.into_iter().map(Into::into);

        let external_services = self.external_services_service.get_external_services_by_ids(ids).await?;
        Ok(external_services.into_iter().map(Into::into).collect())
    }

    async fn all_media(
        &self,
        ctx: &Context<'_>,
        source_ids: Option<Vec<Uuid>>,
        tag_ids: Option<Vec<TagTagTypeInput>>,
        order: Option<OrderDirection>,
        after: Option<String>,
        before: Option<String>,
        #[graphql(validator(maximum = 100))]
        first: Option<i32>,
        #[graphql(validator(maximum = 100))]
        last: Option<i32>,
    ) -> async_graphql::Result<Connection<MediumCursor, Medium>> {
        let tags = ctx.look_ahead().field("edges").field("node").field("tags");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = ctx.look_ahead().field("edges").field("node").field("replicas").exists();
        let sources = ctx.look_ahead().field("edges").field("node").field("sources").exists();

        let source_ids = source_ids.map(|source_ids| source_ids.into_iter().map(Into::into));
        let tag_ids = tag_ids.map(|tag_ids| tag_ids.into_iter().map(Into::into));
        let order = order.unwrap_or(OrderDirection::Asc);

        query(
            after, before, first, last,
            |after, before, first, last| async move {
                let (rev, since, until, limit) = match (first, last) {
                    (None, None) => return Err(QueryError::PaginationRequired)?,
                    (Some(first), _) => match order {
                        OrderDirection::Asc => (false, after, None, first),
                        OrderDirection::Desc => (false, None, after, first),
                    },
                    (_, Some(last)) => match order {
                        OrderDirection::Asc => (true, None, before, last),
                        OrderDirection::Desc => (true, before, None, last),
                    },
                };

                let since = since.map(MediumCursor::into_inner);
                let until = until.map(MediumCursor::into_inner);
                let order = match rev {
                    true => order.rev().into(),
                    false => order.into(),
                };

                let media = {
                    let limit = limit as u64 + 1;
                    match (source_ids, tag_ids) {
                        (Some(_), Some(_)) => return Err(QueryError::MutuallyExclusive("sourceIds", "tagIds"))?,
                        (None, None) => {
                            self.media_service.get_media(tag_depth, replicas, sources, since, until, order, limit).await?
                        },
                        (Some(source_ids), None) => {
                            self.media_service.get_media_by_source_ids(source_ids, tag_depth, replicas, sources, since, until, order, limit).await?
                        },
                        (None, Some(tag_ids)) => {
                            self.media_service.get_media_by_tag_ids(tag_ids, tag_depth, replicas, sources, since, until, order, limit).await?
                        },
                    }
                };

                let has_previous = last.is_some() && media.len() > limit;
                let has_next = last.is_none() && media.len() > limit;
                let media = media.into_iter().take(limit);

                let mut connection = Connection::new(has_previous, has_next);
                let result: anyhow::Result<_> = match rev {
                    true => media.rev().map(TryInto::try_into).collect(),
                    false => media.map(TryInto::try_into).collect(),
                };
                connection.edges = result?;

                anyhow::Ok(connection)
            },
        ).await
    }

    async fn media(&self, ctx: &Context<'_>, ids: Vec<Uuid>) -> anyhow::Result<Vec<Medium>> {
        let tags = ctx.look_ahead().field("tags");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = ctx.look_ahead().field("replicas").exists();
        let sources = ctx.look_ahead().field("sources").exists();

        let ids = ids.into_iter().map(Into::into);

        let media = self.media_service.get_media_by_ids(ids, tag_depth, replicas, sources).await?;
        media.into_iter().map(TryInto::try_into).collect()
    }

    async fn replica(&self, original_url: String) -> anyhow::Result<Replica> {
        let replicas = self.media_service.get_replica_by_original_url(&original_url).await?;
        Ok(replicas.into())
    }

    async fn sources(&self, external_service_id: Uuid, external_metadata: ExternalMetadata) -> anyhow::Result<Vec<Source>> {
        let external_service_id = external_service_id.into();
        let external_metadata = external_metadata.try_into()?;

        let sources = self.media_service.get_sources_by_external_metadata(external_service_id, external_metadata).await?;
        sources.into_iter().map(TryInto::try_into).collect()
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
        let depth = get_tag_depth(&ctx.look_ahead().field("edges").field("node"));

        query(
            after, before, first, last,
            |after, before, first, last| async move {
                let (rev, after, before, limit) = match (first, last) {
                    (Some(first), _) => (false, after, None, first),
                    (_, Some(last)) => (true, None, before, last),
                    (None, None) => return Err(QueryError::PaginationRequired)?,
                };

                let after = after.map(TagCursor::into_inner);
                let before = before.map(TagCursor::into_inner);
                let order = match rev {
                    true => repository::OrderDirection::Descending,
                    false => repository::OrderDirection::Ascending,
                };

                let tags = self.tags_service.get_tags(
                    depth,
                    root,
                    after,
                    before,
                    order,
                    limit as u64 + 1
                ).await?;

                let has_previous = last.is_some() && tags.len() > limit;
                let has_next = last.is_none() && tags.len() > limit;
                let tags = tags.into_iter().take(limit);

                let mut connection = Connection::new(has_previous, has_next);
                connection.edges = match rev {
                    true => tags.rev().map(Into::into).collect(),
                    false => tags.map(Into::into).collect(),
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
        let ids = ids.into_iter().map(Into::into).collect();

        let tags = self.tags_service.get_tags_by_ids(ids, depth).await?;
        Ok(tags.into_iter().map(Into::into).collect())
    }

    async fn all_tag_types(&self) -> anyhow::Result<Vec<TagType>> {
        let tag_types = self.tags_service.get_tag_types().await?;
        Ok(tag_types.into_iter().map(Into::into).collect())
    }
}
