use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{AliasSet, Tag, TagDepth, TagId},
    },
    repository::media::MediaRepository,
};
use futures::TryStreamExt;
use postgres::media::PostgresMediaRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.create(
        [],
        None,
        [],
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
async fn with_created_at_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.create(
        [],
        Some(Utc.with_ymd_and_hms(2022, 1, 1, 5, 6, 7).unwrap()),
        [],
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
    assert_eq!(actual.get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 1, 1, 5, 6, 7).unwrap());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_sources_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.create(
        [
            SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
            SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
        ],
        None,
        [],
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
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 15).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 17).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
            external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
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
async fn with_tags_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.create(
        [],
        None,
        [
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                    name: "船見結衣".to_string(),
                                    kana: "ふなみゆい".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                    name: "吉川ちなつ".to_string(),
                                    kana: "よしかわちなつ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                            name: "魔女っ娘ミラクるん".to_string(),
                            kana: "まじょっこミラクるん".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
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
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                        })),
                        children: Vec::new(),
                        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
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
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                        })),
                        children: Vec::new(),
                        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
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
async fn with_sources_tags_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.create(
        [
            SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
            SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
        ],
        None,
        [
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
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 15).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 17).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
            external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                    name: "船見結衣".to_string(),
                                    kana: "ふなみゆい".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                    name: "吉川ちなつ".to_string(),
                                    kana: "よしかわちなつ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                            name: "魔女っ娘ミラクるん".to_string(),
                            kana: "まじょっこミラクるん".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
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
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                        })),
                        children: Vec::new(),
                        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
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
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                        })),
                        children: Vec::new(),
                        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
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
