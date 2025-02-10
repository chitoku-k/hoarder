use std::{collections::BTreeSet, sync::Arc};

use async_graphql::{Schema, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::entity::{
    external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
    media::{Medium, MediumId},
    sources::{Source, SourceId},
    tag_types::{TagType, TagTypeId},
    tags::{AliasSet, Tag, TagDepth, TagId},
};
use futures::future::ok;
use indoc::indoc;
use ordermap::OrderMap;
use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::{mutation::Mutation, query::Query};

use super::mocks::{
    domain::service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
    normalizer::MockNormalizerInterface,
};

#[tokio::test]
async fn succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_create_medium()
        .times(1)
        .withf(|source_ids, created_at, tag_tag_type_ids, tag_depth, sources| {
            source_ids.clone_box().eq([
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            ]) &&
            tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
                (
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            (created_at, tag_depth, sources) == (
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
                &Some(TagDepth::new(0, 0)),
                &true,
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            slug: "x".to_string(),
                            kind: ExternalServiceKind::X,
                            name: "X".to_string(),
                            base_url: Some("https://x.com".to_string()),
                            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                        },
                        external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
                    },
                    Source {
                        id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                            slug: "pixiv".to_string(),
                            kind: ExternalServiceKind::Pixiv,
                            name: "pixiv".to_string(),
                            base_url: Some("https://www.pixiv.net".to_string()),
                            url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                        created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
                    },
                ],
                tags: {
                    let mut tags = OrderMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                            kana: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                parent: None,
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: Default::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                            kana: "さくひん".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: Default::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
            }))
        });

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        mutation {
            createMedium(
                sourceIds: [
                    "11111111-1111-1111-1111-111111111111",
                    "22222222-2222-2222-2222-222222222222",
                ],
                createdAt: "2022-06-01T12:34:56+00:00",
                tagIds: [
                    {
                        tagId: "33333333-3333-3333-3333-333333333333",
                        tagTypeId: "44444444-4444-4444-4444-444444444444",
                    },
                    {
                        tagId: "55555555-5555-5555-5555-555555555555",
                        tagTypeId: "44444444-4444-4444-4444-444444444444",
                    },
                ],
            ) {
                id
                sources {
                    id
                    externalService {
                        id
                        slug
                        kind
                        name
                        baseUrl
                        urlPattern
                    }
                    externalMetadata
                    url
                    createdAt
                    updatedAt
                }
                tags {
                    tag {
                        id
                        name
                        kana
                        aliases
                        createdAt
                        updatedAt
                    }
                    type {
                        id
                        slug
                        name
                        kana
                    }
                }
                createdAt
                updatedAt
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "createMedium": {
            "id": "77777777-7777-7777-7777-777777777777",
            "sources": [
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "externalService": {
                        "id": "33333333-3333-3333-3333-333333333333",
                        "slug": "x",
                        "kind": "x",
                        "name": "X",
                        "baseUrl": "https://x.com",
                        "urlPattern": r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$",
                    },
                    "externalMetadata": {
                        "x": {
                            "id": "727620202049900544",
                            "creatorId": "_namori_",
                        },
                    },
                    "url": "https://x.com/_namori_/status/727620202049900544",
                    "createdAt": "2016-05-04T07:05:00+00:00",
                    "updatedAt": "2016-05-04T07:05:01+00:00",
                },
                {
                    "id": "22222222-2222-2222-2222-222222222222",
                    "externalService": {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "slug": "pixiv",
                        "kind": "pixiv",
                        "name": "pixiv",
                        "baseUrl": "https://www.pixiv.net",
                        "urlPattern": r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$",
                    },
                    "externalMetadata": {
                        "pixiv": {
                            "id": "56736941",
                        },
                    },
                    "url": "https://www.pixiv.net/artworks/56736941",
                    "createdAt": "2016-05-06T05:14:00+00:00",
                    "updatedAt": "2016-05-06T05:14:01+00:00",
                },
            ],
            "tags": [
                {
                    "tag": {
                        "id": "33333333-3333-3333-3333-333333333333",
                        "name": "赤座あかり",
                        "kana": "あかざあかり",
                        "aliases": ["アッカリーン"],
                        "createdAt": "2022-06-01T00:00:00+00:00",
                        "updatedAt": "2022-06-01T00:01:00+00:00",
                    },
                    "type": {
                        "id": "44444444-4444-4444-4444-444444444444",
                        "slug": "character",
                        "name": "キャラクター",
                        "kana": "キャラクター",
                    },
                },
                {
                    "tag": {
                        "id": "55555555-5555-5555-5555-555555555555",
                        "name": "歳納京子",
                        "kana": "としのうきょうこ",
                        "aliases": [],
                        "createdAt": "2022-06-01T00:02:00+00:00",
                        "updatedAt": "2022-06-01T00:03:00+00:00",
                    },
                    "type": {
                        "id": "44444444-4444-4444-4444-444444444444",
                        "slug": "character",
                        "name": "キャラクター",
                        "kana": "キャラクター",
                    },
                },
                {
                    "tag": {
                        "id": "22222222-2222-2222-2222-222222222222",
                        "name": "ゆるゆり",
                        "kana": "ゆるゆり",
                        "aliases": [],
                        "createdAt": "2022-06-01T00:00:00+00:00",
                        "updatedAt": "2022-06-01T00:01:00+00:00",
                    },
                    "type": {
                        "id": "66666666-6666-6666-6666-666666666666",
                        "slug": "work",
                        "name": "作品",
                        "kana": "さくひん",
                    },
                },
            ],
            "createdAt": "2022-06-01T12:34:56+00:00",
            "updatedAt": "2022-06-01T00:05:00+00:00",
        },
    }));
}
