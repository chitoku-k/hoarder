use std::collections::{BTreeMap, BTreeSet};

use chrono::{NaiveDate, NaiveDateTime};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{AliasSet, Tag, TagDepth, TagId},
    },
    repository::{self, media::MediaRepository, DeleteResult},
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
async fn create_succeeds(ctx: &DatabaseContext) {
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
async fn create_with_created_at_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.create(
        [],
        Some(NaiveDate::from_ymd_opt(2022, 1, 1).and_then(|d| d.and_hms_opt(5, 6, 7)).unwrap()),
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
    assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 1, 1).and_then(|d| d.and_hms_opt(5, 6, 7)).unwrap());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn create_with_sources_succeeds(ctx: &DatabaseContext) {
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
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 15)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 17)).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
            external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                    name: "船見結衣".to_string(),
                                    kana: "ふなみゆい".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                    name: "吉川ちなつ".to_string(),
                                    kana: "よしかわちなつ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                            name: "魔女っ娘ミラクるん".to_string(),
                            kana: "まじょっこミラクるん".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 15)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 17)).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
            external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                    name: "船見結衣".to_string(),
                                    kana: "ふなみゆい".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                    name: "吉川ちなつ".to_string(),
                                    kana: "よしかわちなつ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                            name: "魔女っ娘ミラクるん".to_string(),
                            kana: "まじょっこミラクるん".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                        })),
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
        [
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
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_ids_with_tags_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids(
        [
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_ids_with_replicas_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids(
        [
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                    display_order: Some(2),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: Some(3),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_ids_with_sources_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids(
        [
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 12)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_ids_with_tags_replicas_sources_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids(
        [
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 12)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                    display_order: Some(2),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: Some(3),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        false,
        None,
        None,
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_tags_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        None,
        None,
        repository::OrderDirection::Ascending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_replicas_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        true,
        false,
        None,
        None,
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                    display_order: Some(2),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: Some(3),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_sources_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        None,
        None,
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 12)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        false,
        None,
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_tags_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        None,
        None,
        repository::OrderDirection::Descending,
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_replicas_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        true,
        false,
        None,
        None,
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                    display_order: Some(2),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: Some(3),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_sources_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        None,
        None,
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 12)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_tags_and_since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Ascending,
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_replicas_and_since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        true,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                    display_order: Some(2),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: Some(3),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_sources_and_since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 12)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_tags_and_since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Descending,
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_replicas_and_since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        true,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                    display_order: Some(2),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: Some(3),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_sources_and_since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 12)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_tags_and_until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        repository::OrderDirection::Ascending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_replicas_and_until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        true,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_sources_and_until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_tags_and_until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        repository::OrderDirection::Descending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_replicas_and_until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        true,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_source_ids_with_sources_and_until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
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
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_tags_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
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
        repository::OrderDirection::Ascending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_replicas_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
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
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_sources_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
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
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
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
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_tags_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
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
        repository::OrderDirection::Descending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_replicas_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
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
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_sources_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
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
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 15)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 17)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 16)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
        None,
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_tags_and_since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
        None,
        repository::OrderDirection::Ascending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_replicas_and_since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        true,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
        None,
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_sources_and_since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        true,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
        None,
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 15)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 17)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 16)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_tags_and_since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
        None,
        repository::OrderDirection::Descending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_replicas_and_since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        true,
        false,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
        None,
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_sources_and_since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        true,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
        None,
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 15)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 17)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 16)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_tags_and_until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Ascending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_replicas_and_until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        true,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_sources_and_until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        true,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_tags_and_until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        Some(TagDepth::new(2, 2)),
        false,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Descending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_replicas_and_until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        true,
        false,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_tag_ids_with_sources_and_until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        true,
        None,
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
        repository::OrderDirection::Ascending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 17)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("040d009c-70df-4f55-ae55-df6e5fc57362")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/040d009c-70df-4f55-ae55-df6e5fc57362.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 18)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 11)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 15)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 5)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("e607c6f5-af17-4f65-9868-b3e72f692f4d")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 5555555 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("da2e3cc8-5b12-45fc-b720-815e74fb8fe6")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 6666666 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Ascending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
                    display_order: Some(2),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: Some(3),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        None,
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 15)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 17)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 16)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 12)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 333333333333 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
        None,
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 17)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("040d009c-70df-4f55-ae55-df6e5fc57362")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/040d009c-70df-4f55-ae55-df6e5fc57362.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 18)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 11)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 15)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("e607c6f5-af17-4f65-9868-b3e72f692f4d")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 5555555 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Ascending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Ascending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Descending,
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                            name: "船見結衣".to_string(),
                                            kana: "ふなみゆい".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                        },
                                        Tag {
                                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                            name: "吉川ちなつ".to_string(),
                                            kana: "よしかわちなつ".to_string(),
                                            aliases: AliasSet::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                                    name: "魔女っ娘ミラクるん".to_string(),
                                    kana: "まじょっこミラクるん".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                                },
                            ],
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
                                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        Some((NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        repository::OrderDirection::Descending,
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        name: "Skeb".to_string(),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
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
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 222222222222 },
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 14)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 15)).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
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
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
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
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                            name: "博麗霊夢".to_string(),
                            kana: "はくれいれいむ".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                            name: "フランドール・スカーレット".to_string(),
                            kana: "フランドール・スカーレット".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                            name: "鈴仙・優曇華院・イナバ".to_string(),
                            kana: "れいせん・うどんげいん・いなば".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
        );
        tags
    });
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
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
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            display_order: Some(2),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            display_order: Some(3),
            has_thumbnail: false,
            original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
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
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                slug: "skeb".to_string(),
                name: "Skeb".to_string(),
            },
            external_metadata: ExternalMetadata::Skeb { id: 1111, creator_id: "creator_02".to_string() },
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
        },
    ]);
    assert_eq!(actual.tags, BTreeMap::new());
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ],
        Some(NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap()),
        None,
        false,
        false,
    ).await.unwrap();

    assert_eq!(actual.sources, Vec::new());
    assert_eq!(actual.tags, BTreeMap::new());
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap());

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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ],
        Some(NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap()),
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
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                            name: "博麗霊夢".to_string(),
                            kana: "はくれいれいむ".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                            name: "フランドール・スカーレット".to_string(),
                            kana: "フランドール・スカーレット".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                            name: "鈴仙・優曇華院・イナバ".to_string(),
                            kana: "れいせん・うどんげいん・いなば".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
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
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
            ],
        );
        tags
    });
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap());

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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ],
        Some(NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap()),
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
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            display_order: Some(2),
            has_thumbnail: false,
            original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            display_order: Some(3),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap());

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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ],
        Some(NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap()),
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
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 16)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                slug: "skeb".to_string(),
                name: "Skeb".to_string(),
            },
            external_metadata: ExternalMetadata::Skeb { id: 1111, creator_id: "creator_02".to_string() },
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
        },
    ]);
    assert_eq!(actual.tags, BTreeMap::new());
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap());

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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
        ],
        Some(NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap()),
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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
        ],
        Some(NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap()),
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
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ],
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ],
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
        ],
        Some(NaiveDate::from_ymd_opt(2022, 4, 5).and_then(|d| d.and_hms_opt(6, 7, 8)).unwrap()),
        None,
        false,
        false,
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}
