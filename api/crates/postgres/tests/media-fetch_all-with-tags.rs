use std::collections::{BTreeMap, BTreeSet};

use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::{Medium, MediumId},
        tag_types::{TagType, TagTypeId},
        tags::{AliasSet, Tag, TagDepth, TagId},
    },
    repository::{media::MediaRepository, Direction, Order},
};
use postgres::media::PostgresMediaRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        Some(TagDepth::new(2, 2)),
        false,
        false,
        None,
        Order::Ascending,
        Direction::Forward,
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
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
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        Some(TagDepth::new(2, 2)),
        false,
        false,
        None,
        Order::Descending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        Some(TagDepth::new(2, 2)),
        false,
        false,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Ascending,
        Direction::Forward,
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
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        Some(TagDepth::new(2, 2)),
        false,
        false,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
        Order::Descending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("fdd88e21-fbb0-49e0-854e-54f2cca208f1")),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                            name: "鈴仙・優曇華院・イナバ".to_string(),
                            kana: "れいせん・うどんげいん・いなば".to_string(),
                            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                            parent: Some(Box::new(Tag {
                                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                                name: "東方Project".to_string(),
                                kana: "とうほうProject".to_string(),
                                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                                parent: None,
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        Some(TagDepth::new(2, 2)),
        false,
        false,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Ascending,
        Direction::Backward,
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                })),
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
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
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        Some(TagDepth::new(2, 2)),
        false,
        false,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Descending,
        Direction::Backward,
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                                    name: "博麗霊夢".to_string(),
                                    kana: "はくれいれいむ".to_string(),
                                    aliases: AliasSet::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                                    name: "フランドール・スカーレット".to_string(),
                                    kana: "フランドール・スカーレット".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                                    name: "鈴仙・優曇華院・イナバ".to_string(),
                                    kana: "れいせん・うどんげいん・いなば".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                                },
                            ],
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
                                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                            })),
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
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
            },
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}
