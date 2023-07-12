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
    OrderDirection,
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
        order: OrderDirection,
        after: Option<String>,
        before: Option<String>,
        #[graphql(validator(maximum = 100))]
        first: Option<i32>,
        #[graphql(validator(maximum = 100))]
        last: Option<i32>,
    ) -> async_graphql::Result<Connection<MediumCursor, Medium>> {
        let tags = ctx.look_ahead().field("edges").field("node").field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = ctx.look_ahead().field("edges").field("node").field("replicas").exists();
        let sources = ctx.look_ahead().field("edges").field("node").field("sources").exists();

        query(
            after, before, first, last,
            |after, before, first, last| async move {
                let (rev, limit) = match (first, last) {
                    (None, None) => return Err(QueryError::PaginationRequired)?,
                    (Some(first), _) => (false, first),
                    (_, Some(last)) => (true, last),
                };

                let since = after.map(MediumCursor::into_inner);
                let until = before.map(MediumCursor::into_inner);
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
                            let source_ids: Map<_, _, _> = source_ids.into_iter().map(Into::into);
                            self.media_service.get_media_by_source_ids(source_ids, tag_depth, replicas, sources, since, until, order, limit).await?
                        },
                        (None, Some(tag_ids)) => {
                            let tag_ids: Map<_, _, _> = tag_ids.into_iter().map(Into::into);
                            self.media_service.get_media_by_tag_ids(tag_ids, tag_depth, replicas, sources, since, until, order, limit).await?
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
        let tags = ctx.look_ahead().field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = ctx.look_ahead().field("replicas").exists();
        let sources = ctx.look_ahead().field("sources").exists();

        let ids: Map<_, _, _> = ids.into_iter().map(Into::into);

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
                let (rev, limit) = match (first, last) {
                    (None, None) => return Err(QueryError::PaginationRequired)?,
                    (Some(first), _) => (false, first),
                    (_, Some(last)) => (true, last),
                };

                let since = after.map(TagCursor::into_inner);
                let until = before.map(TagCursor::into_inner);
                let order = match rev {
                    true => repository::OrderDirection::Descending,
                    false => repository::OrderDirection::Ascending,
                };

                let tags = self.tags_service.get_tags(
                    depth,
                    root,
                    since,
                    until,
                    order,
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
    use async_trait::async_trait;
    use chrono::{NaiveDate, NaiveDateTime};
    use domain::{
        entity::{
            external_services::{self, ExternalServiceId},
            media::{self, MediumId},
            replicas::{self, ReplicaId},
            sources::{self, SourceId},
            tag_types::{self, TagTypeId},
            tags::{self, AliasSet, TagDepth, TagId},
        },
        service:: {
            external_services::ExternalServicesServiceInterface,
            media::MediaServiceInterface,
        },
        repository::{DeleteResult, OrderDirection},
    };
    use indoc::indoc;
    use mockall::mock;
    use pretty_assertions::assert_eq;
    use thumbnails::ThumbnailURLFactory;
    use uuid::uuid;

    use super::*;

    mock! {
        ExternalServicesService {}

        #[async_trait]
        impl ExternalServicesServiceInterface for ExternalServicesService {
            async fn create_external_service(&self, slug: &str, name: &str) -> anyhow::Result<external_services::ExternalService>;
            async fn get_external_services(&self) -> anyhow::Result<Vec<external_services::ExternalService>>;
            async fn get_external_services_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<external_services::ExternalService>>
            where
                T: IntoIterator<Item = ExternalServiceId> + Send + Sync + 'static;
            async fn update_external_service_by_id<'a>(&self, id: ExternalServiceId, name: Option<&'a str>) -> anyhow::Result<external_services::ExternalService>;
            async fn delete_external_service_by_id(&self, id: ExternalServiceId) -> anyhow::Result<DeleteResult>;
        }
    }

    mock! {
        MediaService {}

        #[async_trait]
        impl MediaServiceInterface for MediaService {
            async fn create_medium<T, U>(&self, source_ids: T, created_at: Option<NaiveDateTime>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> anyhow::Result<media::Medium>
            where
                T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
                U: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static;
            async fn create_replica(&self, medium_id: MediumId, thumbnail: Option<Vec<u8>>, original_url: &str, mime_type: &str) -> anyhow::Result<replicas::Replica>;
            async fn create_source(&self, external_service_id: ExternalServiceId, external_metadata: external_services::ExternalMetadata) -> anyhow::Result<sources::Source>;
            async fn get_media(&self, tag_depth: Option<TagDepth>, replicas: bool, sources: bool, since: Option<(NaiveDateTime, MediumId)>, until: Option<(NaiveDateTime, MediumId)>, order: OrderDirection, limit: u64) -> anyhow::Result<Vec<media::Medium>>;
            async fn get_media_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> anyhow::Result<Vec<media::Medium>>
            where
                T: IntoIterator<Item = MediumId> + Send + Sync + 'static;
            async fn get_media_by_source_ids<T>(&self, source_ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool, since: Option<(NaiveDateTime, MediumId)>, until: Option<(NaiveDateTime, MediumId)>, order: OrderDirection, limit: u64) -> anyhow::Result<Vec<media::Medium>>
            where
                T: IntoIterator<Item = SourceId> + Send + Sync + 'static;
            async fn get_media_by_tag_ids<T>(&self, tag_tag_type_ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool, since: Option<(NaiveDateTime, MediumId)>, until: Option<(NaiveDateTime, MediumId)>, order: OrderDirection, limit: u64) -> anyhow::Result<Vec<media::Medium>>
            where
                T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static;
            async fn get_replicas_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<replicas::Replica>>
            where
                T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;
            async fn get_replica_by_original_url(&self, original_url: &str) -> anyhow::Result<replicas::Replica>;
            async fn get_sources_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: external_services::ExternalMetadata) -> anyhow::Result<Vec<sources::Source>>;
            async fn get_thumbnail_by_id(&self, id: ReplicaId) -> anyhow::Result<replicas::ReplicaThumbnail>;
            async fn update_medium_by_id<T, U, V, W, X>(&self, id: MediumId, add_source_ids: T, remove_source_ids: U, add_tag_tag_type_ids: V, remove_tag_tag_type_ids: W, replica_orders: X, created_at: Option<NaiveDateTime>, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> anyhow::Result<media::Medium>
            where
                T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
                U: IntoIterator<Item = SourceId> + Send + Sync + 'static,
                V: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
                W: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
                X: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;
            async fn update_replica_by_id<'a, 'b>(&self, id: ReplicaId, thumbnail: Option<Vec<u8>>, original_url: Option<&'a str>, mime_type: Option<&'b str>) -> anyhow::Result<replicas::Replica>;
            async fn update_source_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<external_services::ExternalMetadata>) -> anyhow::Result<sources::Source>;
            async fn delete_medium_by_id(&self, id: MediumId) -> anyhow::Result<DeleteResult>;
            async fn delete_replica_by_id(&self, id: ReplicaId) -> anyhow::Result<DeleteResult>;
            async fn delete_source_by_id(&self, id: SourceId) -> anyhow::Result<DeleteResult>;
        }
    }

    mock! {
        TagsService {}

        #[async_trait]
        impl TagsServiceInterface for TagsService {
            async fn create_tag(&self, name: &str, kana: &str, aliases: &[String], parent_id: Option<TagId>, depth: TagDepth) -> anyhow::Result<tags::Tag>;
            async fn create_tag_type(&self, slug: &str, name: &str) -> anyhow::Result<tag_types::TagType>;
            async fn get_tags(&self, depth: TagDepth, root: bool, after: Option<(String, TagId)>, before: Option<(String, TagId)>, order: OrderDirection, limit: u64) -> anyhow::Result<Vec<tags::Tag>>;
            async fn get_tags_by_ids<T>(&self, ids: T, depth: TagDepth) -> anyhow::Result<Vec<tags::Tag>>
            where
                T: IntoIterator<Item = TagId> + Send + Sync + 'static;
            async fn get_tags_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> anyhow::Result<Vec<tags::Tag>>;
            async fn get_tag_types(&self) -> anyhow::Result<Vec<tag_types::TagType>>;
            async fn update_tag_by_id<T, U>(&self, id: TagId, name: Option<String>, kana: Option<String>, add_aliases: T, remove_aliases: U, depth: TagDepth) -> anyhow::Result<tags::Tag>
            where
                T: IntoIterator<Item = String> + Send + Sync + 'static,
                U: IntoIterator<Item = String> + Send + Sync + 'static;
            async fn update_tag_type_by_id<'a, 'b>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'b str>) -> anyhow::Result<tag_types::TagType>;
            async fn attach_tag_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> anyhow::Result<tags::Tag>;
            async fn detach_tag_by_id(&self, id: TagId, depth: TagDepth) -> anyhow::Result<tags::Tag>;
            async fn delete_tag_by_id(&self, id: TagId, recursive: bool) -> anyhow::Result<DeleteResult>;
            async fn delete_tag_type_by_id(&self, id: TagTypeId) -> anyhow::Result<DeleteResult>;
        }
    }

    // Concrete type is required both in implementation and expectation.
    type IntoIterMap<T, U> = std::iter::Map<std::vec::IntoIter<T>, fn(T) -> U>;

    #[tokio::test]
    async fn all_external_services_succeeds() {
        let mut external_services_service = MockExternalServicesService::new();
        external_services_service
            .expect_get_external_services()
            .times(1)
            .returning(|| {
                Ok(vec![
                    external_services::ExternalService {
                        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_services::ExternalService {
                        id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_services::ExternalService {
                        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                ])
            });

        let media_service = MockMediaService::new();
        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allExternalServices {
                    id
                    slug
                    name
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allExternalServices": [
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "slug": "pixiv",
                    "name": "pixiv",
                },
                {
                    "id": "22222222-2222-2222-2222-222222222222",
                    "slug": "skeb",
                    "name": "Skeb",
                },
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "slug": "twitter",
                    "name": "Twitter",
                },
            ],
        }));
    }

    #[tokio::test]
    async fn external_services_succeeds() {
        let mut external_services_service = MockExternalServicesService::new();
        external_services_service
            .expect_get_external_services_by_ids::<IntoIterMap<Uuid, ExternalServiceId>>()
            .times(1)
            .withf(|ids| ids.clone().eq([
                ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ]))
            .returning(|_| {
                Ok(vec![
                    external_services::ExternalService {
                        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_services::ExternalService {
                        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                ])
            });

        let media_service = MockMediaService::new();
        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                externalServices(ids: ["11111111-1111-1111-1111-111111111111", "33333333-3333-3333-3333-333333333333"]) {
                    id
                    slug
                    name
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "externalServices": [
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "slug": "pixiv",
                    "name": "pixiv",
                },
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "slug": "twitter",
                    "name": "Twitter",
                },
            ],
        }));
    }

    #[tokio::test]
    async fn all_media_with_tags_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &Some(TagDepth::new(2, 2)),
                    &false,
                    &false,
                    &None,
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: {
                            let mut tags = BTreeMap::new();
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                    slug: "character".to_string(),
                                    name: "キャラクター".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                        name: "赤座あかり".to_string(),
                                        kana: "あかざあかり".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                        parent: Some(Box::new(tags::Tag {
                                            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                            name: "ゆるゆり".to_string(),
                                            kana: "ゆるゆり".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        })),
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                    tags::Tag {
                                        id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                        name: "歳納京子".to_string(),
                                        kana: "としのうきょうこ".to_string(),
                                        aliases: Default::default(),
                                        parent: Some(Box::new(tags::Tag {
                                            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                            name: "ゆるゆり".to_string(),
                                            kana: "ゆるゆり".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        })),
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                    slug: "work".to_string(),
                                    name: "作品".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: vec![
                                            tags::Tag {
                                                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                            },
                                            tags::Tag {
                                                id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: Default::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags
                        },
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: {
                            let mut tags = BTreeMap::new();
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                    slug: "character".to_string(),
                                    name: "キャラクター".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                        name: "歳納京子".to_string(),
                                        kana: "としのうきょうこ".to_string(),
                                        aliases: Default::default(),
                                        parent: Some(Box::new(tags::Tag {
                                            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                            name: "ゆるゆり".to_string(),
                                            kana: "ゆるゆり".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        })),
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                    slug: "work".to_string(),
                                    name: "作品".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: vec![
                                            tags::Tag {
                                                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                            },
                                            tags::Tag {
                                                id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: Default::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags
                        },
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: {
                            let mut tags = BTreeMap::new();
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                    slug: "character".to_string(),
                                    name: "キャラクター".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                        name: "赤座あかり".to_string(),
                                        kana: "あかざあかり".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                        parent: Some(Box::new(tags::Tag {
                                            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                            name: "ゆるゆり".to_string(),
                                            kana: "ゆるゆり".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        })),
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                    slug: "work".to_string(),
                                    name: "作品".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: vec![
                                            tags::Tag {
                                                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                            },
                                            tags::Tag {
                                                id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: Default::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags
                        },
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            tags {
                                tag {
                                    id
                                    name
                                    kana
                                    aliases
                                    parent {
                                        id
                                        name
                                        kana
                                        aliases
                                        parent {
                                            id
                                            name
                                            kana
                                            aliases
                                            createdAt
                                            updatedAt
                                        }
                                        createdAt
                                        updatedAt
                                    }
                                    children {
                                        id
                                        name
                                        kana
                                        aliases
                                        children {
                                            id
                                            name
                                            kana
                                            aliases
                                            createdAt
                                            updatedAt
                                        }
                                        createdAt
                                        updatedAt
                                    }
                                    createdAt
                                    updatedAt
                                }
                                type {
                                    id
                                    slug
                                    name
                                }
                            }
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": true,
                },
                "edges": [
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "tags": [
                                {
                                    "tag": {
                                        "id": "33333333-3333-3333-3333-333333333333",
                                        "name": "赤座あかり",
                                        "kana": "あかざあかり",
                                        "aliases": ["アッカリーン"],
                                        "parent": {
                                            "id": "22222222-2222-2222-2222-222222222222",
                                            "name": "ゆるゆり",
                                            "kana": "ゆるゆり",
                                            "aliases": [],
                                            "parent": null,
                                            "createdAt": "2022-06-01T00:00:00",
                                            "updatedAt": "2022-06-01T00:01:00",
                                        },
                                        "children": [],
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "type": {
                                        "id": "44444444-4444-4444-4444-444444444444",
                                        "slug": "character",
                                        "name": "キャラクター",
                                    },
                                },
                                {
                                    "tag": {
                                        "id": "55555555-5555-5555-5555-555555555555",
                                        "name": "歳納京子",
                                        "kana": "としのうきょうこ",
                                        "aliases": [],
                                        "parent": {
                                            "id": "22222222-2222-2222-2222-222222222222",
                                            "name": "ゆるゆり",
                                            "kana": "ゆるゆり",
                                            "aliases": [],
                                            "parent": null,
                                            "createdAt": "2022-06-01T00:00:00",
                                            "updatedAt": "2022-06-01T00:01:00",
                                        },
                                        "children": [],
                                        "createdAt": "2022-06-01T00:02:00",
                                        "updatedAt": "2022-06-01T00:03:00",
                                    },
                                    "type": {
                                        "id": "44444444-4444-4444-4444-444444444444",
                                        "slug": "character",
                                        "name": "キャラクター",
                                    },
                                },
                                {
                                    "tag": {
                                        "id": "22222222-2222-2222-2222-222222222222",
                                        "name": "ゆるゆり",
                                        "kana": "ゆるゆり",
                                        "aliases": [],
                                        "parent": null,
                                        "children": [
                                            {
                                                "id": "33333333-3333-3333-3333-333333333333",
                                                "name": "赤座あかり",
                                                "kana": "あかざあかり",
                                                "aliases": ["アッカリーン"],
                                                "children": [],
                                                "createdAt": "2022-06-01T00:00:00",
                                                "updatedAt": "2022-06-01T00:01:00",
                                            },
                                            {
                                                "id": "55555555-5555-5555-5555-555555555555",
                                                "name": "歳納京子",
                                                "kana": "としのうきょうこ",
                                                "aliases": [],
                                                "children": [],
                                                "createdAt": "2022-06-01T00:02:00",
                                                "updatedAt": "2022-06-01T00:03:00",
                                            },
                                        ],
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "type": {
                                        "id": "66666666-6666-6666-6666-666666666666",
                                        "slug": "work",
                                        "name": "作品",
                                    },
                                },
                            ],
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "tags": [
                                {
                                    "tag": {
                                        "id": "55555555-5555-5555-5555-555555555555",
                                        "name": "歳納京子",
                                        "kana": "としのうきょうこ",
                                        "aliases": [],
                                        "parent": {
                                            "id": "22222222-2222-2222-2222-222222222222",
                                            "name": "ゆるゆり",
                                            "kana": "ゆるゆり",
                                            "aliases": [],
                                            "parent": null,
                                            "createdAt": "2022-06-01T00:00:00",
                                            "updatedAt": "2022-06-01T00:01:00",
                                        },
                                        "children": [],
                                        "createdAt": "2022-06-01T00:02:00",
                                        "updatedAt": "2022-06-01T00:03:00",
                                    },
                                    "type": {
                                        "id": "44444444-4444-4444-4444-444444444444",
                                        "slug": "character",
                                        "name": "キャラクター",
                                    },
                                },
                                {
                                    "tag": {
                                        "id": "22222222-2222-2222-2222-222222222222",
                                        "name": "ゆるゆり",
                                        "kana": "ゆるゆり",
                                        "aliases": [],
                                        "parent": null,
                                        "children": [
                                            {
                                                "id": "33333333-3333-3333-3333-333333333333",
                                                "name": "赤座あかり",
                                                "kana": "あかざあかり",
                                                "aliases": ["アッカリーン"],
                                                "children": [],
                                                "createdAt": "2022-06-01T00:00:00",
                                                "updatedAt": "2022-06-01T00:01:00",
                                            },
                                            {
                                                "id": "55555555-5555-5555-5555-555555555555",
                                                "name": "歳納京子",
                                                "kana": "としのうきょうこ",
                                                "aliases": [],
                                                "children": [],
                                                "createdAt": "2022-06-01T00:02:00",
                                                "updatedAt": "2022-06-01T00:03:00",
                                            },
                                        ],
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "type": {
                                        "id": "66666666-6666-6666-6666-666666666666",
                                        "slug": "work",
                                        "name": "作品",
                                    },
                                },
                            ],
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "tags": [
                                {
                                    "tag": {
                                        "id": "33333333-3333-3333-3333-333333333333",
                                        "name": "赤座あかり",
                                        "kana": "あかざあかり",
                                        "aliases": ["アッカリーン"],
                                        "parent": {
                                            "id": "22222222-2222-2222-2222-222222222222",
                                            "name": "ゆるゆり",
                                            "kana": "ゆるゆり",
                                            "aliases": [],
                                            "parent": null,
                                            "createdAt": "2022-06-01T00:00:00",
                                            "updatedAt": "2022-06-01T00:01:00",
                                        },
                                        "children": [],
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "type": {
                                        "id": "44444444-4444-4444-4444-444444444444",
                                        "slug": "character",
                                        "name": "キャラクター",
                                    },
                                },
                                {
                                    "tag": {
                                        "id": "22222222-2222-2222-2222-222222222222",
                                        "name": "ゆるゆり",
                                        "kana": "ゆるゆり",
                                        "aliases": [],
                                        "parent": null,
                                        "children": [
                                            {
                                                "id": "33333333-3333-3333-3333-333333333333",
                                                "name": "赤座あかり",
                                                "kana": "あかざあかり",
                                                "aliases": ["アッカリーン"],
                                                "children": [],
                                                "createdAt": "2022-06-01T00:00:00",
                                                "updatedAt": "2022-06-01T00:01:00",
                                            },
                                            {
                                                "id": "55555555-5555-5555-5555-555555555555",
                                                "name": "歳納京子",
                                                "kana": "としのうきょうこ",
                                                "aliases": [],
                                                "children": [],
                                                "createdAt": "2022-06-01T00:02:00",
                                                "updatedAt": "2022-06-01T00:03:00",
                                            },
                                        ],
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "type": {
                                        "id": "66666666-6666-6666-6666-666666666666",
                                        "slug": "work",
                                        "name": "作品",
                                    },
                                },
                            ],
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_with_replicas_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &true,
                    &false,
                    &None,
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: vec![
                            replicas::Replica {
                                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                display_order: Some(1),
                                has_thumbnail: true,
                                original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                                mime_type: "image/png".to_string(),
                                created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                            },
                            replicas::Replica {
                                id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                                display_order: Some(2),
                                has_thumbnail: true,
                                original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                                mime_type: "image/png".to_string(),
                                created_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: vec![
                            replicas::Replica {
                                id: ReplicaId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                                display_order: Some(1),
                                has_thumbnail: false,
                                original_url: "file:///var/lib/hoarder/88888888-8888-8888-8888-888888888888.png".to_string(),
                                mime_type: "image/png".to_string(),
                                created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                            },
                            replicas::Replica {
                                id: ReplicaId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                                display_order: Some(2),
                                has_thumbnail: false,
                                original_url: "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.png".to_string(),
                                mime_type: "image/png".to_string(),
                                created_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription)
            .data(ThumbnailURLFactory::new("https://img.example.com/".to_string()))
            .finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            replicas {
                                id
                                displayOrder
                                originalUrl
                                thumbnailUrl
                                mimeType
                                createdAt
                                updatedAt
                            }
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": true,
                },
                "edges": [
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "replicas": [
                                {
                                    "id": "66666666-6666-6666-6666-666666666666",
                                    "displayOrder": 1,
                                    "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                                    "thumbnailUrl": "https://img.example.com/66666666-6666-6666-6666-666666666666",
                                    "mimeType": "image/png",
                                    "createdAt": "2022-06-02T00:00:00",
                                    "updatedAt": "2022-06-02T00:01:00",
                                },
                                {
                                    "id": "77777777-7777-7777-7777-777777777777",
                                    "displayOrder": 2,
                                    "originalUrl": "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png",
                                    "thumbnailUrl": "https://img.example.com/77777777-7777-7777-7777-777777777777",
                                    "mimeType": "image/png",
                                    "createdAt": "2022-06-03T00:02:00",
                                    "updatedAt": "2022-06-03T00:03:00",
                                },
                            ],
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "replicas": [
                                {
                                    "id": "88888888-8888-8888-8888-888888888888",
                                    "displayOrder": 1,
                                    "originalUrl": "file:///var/lib/hoarder/88888888-8888-8888-8888-888888888888.png",
                                    "thumbnailUrl": null,
                                    "mimeType": "image/png",
                                    "createdAt": "2022-06-02T00:00:00",
                                    "updatedAt": "2022-06-02T00:01:00",
                                },
                                {
                                    "id": "99999999-9999-9999-9999-999999999999",
                                    "displayOrder": 2,
                                    "originalUrl": "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.png",
                                    "thumbnailUrl": null,
                                    "mimeType": "image/png",
                                    "createdAt": "2022-06-03T00:02:00",
                                    "updatedAt": "2022-06-03T00:03:00",
                                },
                            ],
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "replicas": [],
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_with_sources_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &true,
                    &None,
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: vec![
                            sources::Source {
                                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                external_service: external_services::ExternalService {
                                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                    slug: "twitter".to_string(),
                                    name: "Twitter".to_string(),
                                },
                                external_metadata: external_services::ExternalMetadata::Twitter { id: 727620202049900544 },
                                created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
                            },
                            sources::Source {
                                id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                external_service: external_services::ExternalService {
                                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                    slug: "pixiv".to_string(),
                                    name: "pixiv".to_string(),
                                },
                                external_metadata: external_services::ExternalMetadata::Pixiv { id: 56736941 },
                                created_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 1)).unwrap(),
                            },
                        ],
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: vec![
                            sources::Source {
                                id: SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                external_service: external_services::ExternalService {
                                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                    slug: "pixiv".to_string(),
                                    name: "pixiv".to_string(),
                                },
                                external_metadata: external_services::ExternalMetadata::Pixiv { id: 1234 },
                                created_at: NaiveDate::from_ymd_opt(2016, 5, 5).and_then(|d| d.and_hms_opt(7, 6, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2016, 5, 5).and_then(|d| d.and_hms_opt(7, 6, 1)).unwrap(),
                            },
                        ],
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            sources {
                                id
                                externalService {
                                    id
                                    slug
                                    name
                                }
                                externalMetadata
                                createdAt
                                updatedAt
                            }
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": true,
                },
                "edges": [
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "sources": [
                                {
                                    "id": "11111111-1111-1111-1111-111111111111",
                                    "externalService": {
                                        "id": "33333333-3333-3333-3333-333333333333",
                                        "slug": "twitter",
                                        "name": "Twitter",
                                    },
                                    "externalMetadata": {
                                        "twitter": {
                                            "id": "727620202049900544",
                                        },
                                    },
                                    "createdAt": "2016-05-04T07:05:00",
                                    "updatedAt": "2016-05-04T07:05:01",
                                },
                                {
                                    "id": "22222222-2222-2222-2222-222222222222",
                                    "externalService": {
                                        "id": "11111111-1111-1111-1111-111111111111",
                                        "slug": "pixiv",
                                        "name": "pixiv",
                                    },
                                    "externalMetadata": {
                                        "pixiv": {
                                            "id": "56736941",
                                        },
                                    },
                                    "createdAt": "2016-05-06T05:14:00",
                                    "updatedAt": "2016-05-06T05:14:01",
                                },
                            ],
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "sources": [
                                {
                                    "id": "33333333-3333-3333-3333-333333333333",
                                    "externalService": {
                                        "id": "11111111-1111-1111-1111-111111111111",
                                        "slug": "pixiv",
                                        "name": "pixiv",
                                    },
                                    "externalMetadata": {
                                        "pixiv": {
                                            "id": "1234",
                                        },
                                    },
                                    "createdAt": "2016-05-05T07:06:00",
                                    "updatedAt": "2016-05-05T07:06:01",
                                },
                            ],
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "sources": [],
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_first_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": true,
                },
                "edges": [
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_first_desc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &None,
                    &OrderDirection::Descending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3, order: DESC) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": true,
                },
                "edges": [
                    {
                        "node": {
                            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "createdAt": "2022-06-01T12:34:59",
                            "updatedAt": "2022-06-01T00:05:03",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_first_after_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3, after: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                    {
                        "node": {
                            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "createdAt": "2022-06-01T12:34:59",
                            "updatedAt": "2022-06-01T00:05:03",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_first_after_desc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                    &None,
                    &OrderDirection::Descending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3, order: DESC, after: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_first_before_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3, before: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_first_before_desc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                    &OrderDirection::Descending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3, order: DESC, before: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "createdAt": "2022-06-01T12:34:59",
                            "updatedAt": "2022-06-01T00:05:03",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_last_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &None,
                    &OrderDirection::Descending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(last: 3, order: ASC) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": true,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                    {
                        "node": {
                            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "createdAt": "2022-06-01T12:34:59",
                            "updatedAt": "2022-06-01T00:05:03",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_last_desc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(last: 3, order: DESC) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": true,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_last_after_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                    &None,
                    &OrderDirection::Descending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(last: 3, after: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                    {
                        "node": {
                            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "createdAt": "2022-06-01T12:34:59",
                            "updatedAt": "2022-06-01T00:05:03",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_last_after_desc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(last: 3, order: DESC, after: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_last_before_asc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                    &OrderDirection::Descending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(last: 3, before: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_last_before_desc_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(last: 3, order: DESC, before: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "createdAt": "2022-06-01T12:34:59",
                            "updatedAt": "2022-06-01T00:05:03",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_by_source_ids_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media_by_source_ids::<IntoIterMap<Uuid, SourceId>>()
            .times(1)
            .withf(|source_ids, tag_depth, replicas, sources, since, until, order, limit| {
                source_ids.clone().eq([
                    SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                ]) &&
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(first: 3, sourceIds: ["11111111-1111-1111-1111-111111111111", "33333333-3333-3333-3333-333333333333"]) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn all_media_by_tag_ids_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media_by_tag_ids::<IntoIterMap<TagTagTypeInput, (TagId, TagTypeId)>>()
            .times(1)
            .withf(|tag_ids, tag_depth, replicas, sources, since, until, order, limit| {
                tag_ids.clone().eq([
                    (
                        TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ]) &&
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &None,
                    &false,
                    &false,
                    &None,
                    &None,
                    &OrderDirection::Ascending,
                    &4,
                )
            })
            .returning(|_, _, _, _, _, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                allMedia(
                    first: 3,
                    tagIds: [
                        {
                            tagId: "22222222-2222-2222-2222-222222222222",
                            tagTypeId: "44444444-4444-4444-4444-444444444444",
                        },
                    ],
                ) {
                    pageInfo {
                        hasPreviousPage
                        hasNextPage
                    }
                    edges {
                        node {
                            id
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "allMedia": {
                "pageInfo": {
                    "hasPreviousPage": false,
                    "hasNextPage": false,
                },
                "edges": [
                    {
                        "node": {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "createdAt": "2022-06-01T12:34:56",
                            "updatedAt": "2022-06-01T00:05:00",
                        },
                    },
                    {
                        "node": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "createdAt": "2022-06-01T12:34:57",
                            "updatedAt": "2022-06-01T00:05:01",
                        },
                    },
                    {
                        "node": {
                            "id": "99999999-9999-9999-9999-999999999999",
                            "createdAt": "2022-06-01T12:34:58",
                            "updatedAt": "2022-06-01T00:05:02",
                        },
                    },
                ],
            },
        }));
    }

    #[tokio::test]
    async fn media_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media_by_ids::<IntoIterMap<Uuid, MediumId>>()
            .times(1)
            .withf(|ids, tag_depth, replicas, sources| {
                ids.clone().eq([
                    MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                ]) &&
                (tag_depth, replicas, sources) == (
                    &None,
                    &false,
                    &false,
                )
            })
            .returning(|_, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                    id
                    createdAt
                    updatedAt
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "media": [
                {
                    "id": "77777777-7777-7777-7777-777777777777",
                    "createdAt": "2022-06-01T12:34:56",
                    "updatedAt": "2022-06-01T00:05:00",
                },
                {
                    "id": "99999999-9999-9999-9999-999999999999",
                    "createdAt": "2022-06-01T12:34:58",
                    "updatedAt": "2022-06-01T00:05:02",
                },
            ],
        }));
    }

    #[tokio::test]
    async fn media_with_tags_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media_by_ids::<IntoIterMap<Uuid, MediumId>>()
            .times(1)
            .withf(|ids, tag_depth, replicas, sources| {
                ids.clone().eq([
                    MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                ]) &&
                (tag_depth, replicas, sources) == (
                    &Some(TagDepth::new(2, 2)),
                    &false,
                    &false,
                )
            })
            .returning(|_, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: {
                            let mut tags = BTreeMap::new();
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                    slug: "character".to_string(),
                                    name: "キャラクター".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                        name: "赤座あかり".to_string(),
                                        kana: "あかざあかり".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                        parent: Some(Box::new(tags::Tag {
                                            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                            name: "ゆるゆり".to_string(),
                                            kana: "ゆるゆり".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        })),
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                    tags::Tag {
                                        id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                        name: "歳納京子".to_string(),
                                        kana: "としのうきょうこ".to_string(),
                                        aliases: Default::default(),
                                        parent: Some(Box::new(tags::Tag {
                                            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                            name: "ゆるゆり".to_string(),
                                            kana: "ゆるゆり".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        })),
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                    slug: "work".to_string(),
                                    name: "作品".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: vec![
                                            tags::Tag {
                                                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                            },
                                            tags::Tag {
                                                id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: Default::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags
                        },
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: {
                            let mut tags = BTreeMap::new();
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                    slug: "character".to_string(),
                                    name: "キャラクター".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                        name: "赤座あかり".to_string(),
                                        kana: "あかざあかり".to_string(),
                                        aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                        parent: Some(Box::new(tags::Tag {
                                            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                            name: "ゆるゆり".to_string(),
                                            kana: "ゆるゆり".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        })),
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags.insert(
                                tag_types::TagType {
                                    id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                    slug: "work".to_string(),
                                    name: "作品".to_string(),
                                },
                                vec![
                                    tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: vec![
                                            tags::Tag {
                                                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                                name: "赤座あかり".to_string(),
                                                kana: "あかざあかり".to_string(),
                                                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                            },
                                            tags::Tag {
                                                id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                                name: "歳納京子".to_string(),
                                                kana: "としのうきょうこ".to_string(),
                                                aliases: Default::default(),
                                                parent: None,
                                                children: Vec::new(),
                                                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                            },
                                        ],
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    },
                                ],
                            );
                            tags
                        },
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                    id
                    tags {
                        tag {
                            id
                            name
                            kana
                            aliases
                            parent {
                                id
                                name
                                kana
                                aliases
                                parent {
                                    id
                                    name
                                    kana
                                    aliases
                                    createdAt
                                    updatedAt
                                }
                                createdAt
                                updatedAt
                            }
                            children {
                                id
                                name
                                kana
                                aliases
                                children {
                                    id
                                    name
                                    kana
                                    aliases
                                    createdAt
                                    updatedAt
                                }
                                createdAt
                                updatedAt
                            }
                            createdAt
                            updatedAt
                        }
                        type {
                            id
                            slug
                            name
                        }
                    }
                    createdAt
                    updatedAt
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "media": [
                {
                    "id": "77777777-7777-7777-7777-777777777777",
                    "tags": [
                        {
                            "tag": {
                                "id": "33333333-3333-3333-3333-333333333333",
                                "name": "赤座あかり",
                                "kana": "あかざあかり",
                                "aliases": ["アッカリーン"],
                                "parent": {
                                    "id": "22222222-2222-2222-2222-222222222222",
                                    "name": "ゆるゆり",
                                    "kana": "ゆるゆり",
                                    "aliases": [],
                                    "parent": null,
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
                                },
                                "children": [],
                                "createdAt": "2022-06-01T00:00:00",
                                "updatedAt": "2022-06-01T00:01:00",
                            },
                            "type": {
                                "id": "44444444-4444-4444-4444-444444444444",
                                "slug": "character",
                                "name": "キャラクター",
                            },
                        },
                        {
                            "tag": {
                                "id": "55555555-5555-5555-5555-555555555555",
                                "name": "歳納京子",
                                "kana": "としのうきょうこ",
                                "aliases": [],
                                "parent": {
                                    "id": "22222222-2222-2222-2222-222222222222",
                                    "name": "ゆるゆり",
                                    "kana": "ゆるゆり",
                                    "aliases": [],
                                    "parent": null,
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
                                },
                                "children": [],
                                "createdAt": "2022-06-01T00:02:00",
                                "updatedAt": "2022-06-01T00:03:00",
                            },
                            "type": {
                                "id": "44444444-4444-4444-4444-444444444444",
                                "slug": "character",
                                "name": "キャラクター",
                            },
                        },
                        {
                            "tag": {
                                "id": "22222222-2222-2222-2222-222222222222",
                                "name": "ゆるゆり",
                                "kana": "ゆるゆり",
                                "aliases": [],
                                "parent": null,
                                "children": [
                                    {
                                        "id": "33333333-3333-3333-3333-333333333333",
                                        "name": "赤座あかり",
                                        "kana": "あかざあかり",
                                        "aliases": ["アッカリーン"],
                                        "children": [],
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    {
                                        "id": "55555555-5555-5555-5555-555555555555",
                                        "name": "歳納京子",
                                        "kana": "としのうきょうこ",
                                        "aliases": [],
                                        "children": [],
                                        "createdAt": "2022-06-01T00:02:00",
                                        "updatedAt": "2022-06-01T00:03:00",
                                    },
                                ],
                                "createdAt": "2022-06-01T00:00:00",
                                "updatedAt": "2022-06-01T00:01:00",
                            },
                            "type": {
                                "id": "66666666-6666-6666-6666-666666666666",
                                "slug": "work",
                                "name": "作品",
                            },
                        },
                    ],
                    "createdAt": "2022-06-01T12:34:56",
                    "updatedAt": "2022-06-01T00:05:00",
                },
                {
                    "id": "99999999-9999-9999-9999-999999999999",
                    "tags": [
                        {
                            "tag": {
                                "id": "33333333-3333-3333-3333-333333333333",
                                "name": "赤座あかり",
                                "kana": "あかざあかり",
                                "aliases": ["アッカリーン"],
                                "parent": {
                                    "id": "22222222-2222-2222-2222-222222222222",
                                    "name": "ゆるゆり",
                                    "kana": "ゆるゆり",
                                    "aliases": [],
                                    "parent": null,
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
                                },
                                "children": [],
                                "createdAt": "2022-06-01T00:00:00",
                                "updatedAt": "2022-06-01T00:01:00",
                            },
                            "type": {
                                "id": "44444444-4444-4444-4444-444444444444",
                                "slug": "character",
                                "name": "キャラクター",
                            },
                        },
                        {
                            "tag": {
                                "id": "22222222-2222-2222-2222-222222222222",
                                "name": "ゆるゆり",
                                "kana": "ゆるゆり",
                                "aliases": [],
                                "parent": null,
                                "children": [
                                    {
                                        "id": "33333333-3333-3333-3333-333333333333",
                                        "name": "赤座あかり",
                                        "kana": "あかざあかり",
                                        "aliases": ["アッカリーン"],
                                        "children": [],
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    {
                                        "id": "55555555-5555-5555-5555-555555555555",
                                        "name": "歳納京子",
                                        "kana": "としのうきょうこ",
                                        "aliases": [],
                                        "children": [],
                                        "createdAt": "2022-06-01T00:02:00",
                                        "updatedAt": "2022-06-01T00:03:00",
                                    },
                                ],
                                "createdAt": "2022-06-01T00:00:00",
                                "updatedAt": "2022-06-01T00:01:00",
                            },
                            "type": {
                                "id": "66666666-6666-6666-6666-666666666666",
                                "slug": "work",
                                "name": "作品",
                            },
                        },
                    ],
                    "createdAt": "2022-06-01T12:34:58",
                    "updatedAt": "2022-06-01T00:05:02",
                },
            ],
        }));
    }

    #[tokio::test]
    async fn media_with_replicas_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media_by_ids::<IntoIterMap<Uuid, MediumId>>()
            .times(1)
            .withf(|ids, tag_depth, replicas, sources| {
                ids.clone().eq([
                    MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                ]) &&
                (tag_depth, replicas, sources) == (
                    &None,
                    &true,
                    &false,
                )
            })
            .returning(|_, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: vec![
                            replicas::Replica {
                                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                display_order: Some(1),
                                has_thumbnail: true,
                                original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                                mime_type: "image/png".to_string(),
                                created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                            },
                            replicas::Replica {
                                id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                                display_order: Some(2),
                                has_thumbnail: true,
                                original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                                mime_type: "image/png".to_string(),
                                created_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription)
            .data(ThumbnailURLFactory::new("https://img.example.com/".to_string()))
            .finish();
        let req = indoc! {r#"
            query {
                media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                    id
                    replicas {
                        id
                        displayOrder
                        originalUrl
                        thumbnailUrl
                        mimeType
                        createdAt
                        updatedAt
                    }
                    createdAt
                    updatedAt
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "media": [
                {
                    "id": "77777777-7777-7777-7777-777777777777",
                    "replicas": [
                        {
                            "id": "66666666-6666-6666-6666-666666666666",
                            "displayOrder": 1,
                            "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                            "thumbnailUrl": "https://img.example.com/66666666-6666-6666-6666-666666666666",
                            "mimeType": "image/png",
                            "createdAt": "2022-06-02T00:00:00",
                            "updatedAt": "2022-06-02T00:01:00",
                        },
                        {
                            "id": "77777777-7777-7777-7777-777777777777",
                            "displayOrder": 2,
                            "originalUrl": "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png",
                            "thumbnailUrl": "https://img.example.com/77777777-7777-7777-7777-777777777777",
                            "mimeType": "image/png",
                            "createdAt": "2022-06-03T00:02:00",
                            "updatedAt": "2022-06-03T00:03:00",
                        },
                    ],
                    "createdAt": "2022-06-01T12:34:56",
                    "updatedAt": "2022-06-01T00:05:00",
                },
                {
                    "id": "99999999-9999-9999-9999-999999999999",
                    "replicas": [],
                    "createdAt": "2022-06-01T12:34:58",
                    "updatedAt": "2022-06-01T00:05:02",
                },
            ],
        }));
    }

    #[tokio::test]
    async fn media_with_sources_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_media_by_ids::<IntoIterMap<Uuid, MediumId>>()
            .times(1)
            .withf(|ids, tag_depth, replicas, sources| {
                ids.clone().eq([
                    MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                ]) &&
                (tag_depth, replicas, sources) == (
                    &None,
                    &false,
                    &true,
                )
            })
            .returning(|_, _, _, _| {
                Ok(vec![
                    media::Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: vec![
                            sources::Source {
                                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                external_service: external_services::ExternalService {
                                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                    slug: "twitter".to_string(),
                                    name: "Twitter".to_string(),
                                },
                                external_metadata: external_services::ExternalMetadata::Twitter { id: 727620202049900544 },
                                created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
                            },
                            sources::Source {
                                id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                external_service: external_services::ExternalService {
                                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                    slug: "pixiv".to_string(),
                                    name: "pixiv".to_string(),
                                },
                                external_metadata: external_services::ExternalMetadata::Pixiv { id: 56736941 },
                                created_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 0)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 1)).unwrap(),
                            },
                        ],
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    media::Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
        let req = indoc! {r#"
            query {
                media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                    id
                    sources {
                        id
                        externalService {
                            id
                            slug
                            name
                        }
                        externalMetadata
                        createdAt
                        updatedAt
                    }
                    createdAt
                    updatedAt
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "media": [
                {
                    "id": "77777777-7777-7777-7777-777777777777",
                    "sources": [
                        {
                            "id": "11111111-1111-1111-1111-111111111111",
                            "externalService": {
                                "id": "33333333-3333-3333-3333-333333333333",
                                "slug": "twitter",
                                "name": "Twitter",
                            },
                            "externalMetadata": {
                                "twitter": {
                                    "id": "727620202049900544",
                                },
                            },
                            "createdAt": "2016-05-04T07:05:00",
                            "updatedAt": "2016-05-04T07:05:01",
                        },
                        {
                            "id": "22222222-2222-2222-2222-222222222222",
                            "externalService": {
                                "id": "11111111-1111-1111-1111-111111111111",
                                "slug": "pixiv",
                                "name": "pixiv",
                            },
                            "externalMetadata": {
                                "pixiv": {
                                    "id": "56736941",
                                },
                            },
                            "createdAt": "2016-05-06T05:14:00",
                            "updatedAt": "2016-05-06T05:14:01",
                        },
                    ],
                    "createdAt": "2022-06-01T12:34:56",
                    "updatedAt": "2022-06-01T00:05:00",
                },
                {
                    "id": "99999999-9999-9999-9999-999999999999",
                    "sources": [],
                    "createdAt": "2022-06-01T12:34:58",
                    "updatedAt": "2022-06-01T00:05:02",
                },
            ],
        }));
    }

    #[tokio::test]
    async fn replica_succeeds() {
        let external_services_service = MockExternalServicesService::new();

        let mut media_service = MockMediaService::new();
        media_service
            .expect_get_replica_by_original_url()
            .times(1)
            .withf(|original_url| original_url == "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png")
            .returning(|_| {
                Ok(replicas::Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: Some(1),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                })
            });

        let tags_service = MockTagsService::new();

        let query = Query::new(external_services_service, media_service, tags_service);
        let schema = Schema::build(query, EmptyMutation, EmptySubscription)
            .data(ThumbnailURLFactory::new("https://img.example.com/".to_string()))
            .finish();
        let req = indoc! {r#"
            query {
                replica(originalUrl: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png") {
                    id
                    displayOrder
                    originalUrl
                    thumbnailUrl
                    mimeType
                    createdAt
                    updatedAt
                }
            }
        "#};
        let actual = schema.execute(req).await.into_result().unwrap();

        assert_eq!(actual.data, value!({
            "replica": {
                "id": "66666666-6666-6666-6666-666666666666",
                "displayOrder": 1,
                "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                "thumbnailUrl": "https://img.example.com/66666666-6666-6666-6666-666666666666",
                "mimeType": "image/png",
                "createdAt": "2022-06-02T00:00:00",
                "updatedAt": "2022-06-02T00:01:00",
            },
        }));
    }
}
