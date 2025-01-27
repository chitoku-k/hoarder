use std::collections::BTreeSet;

use domain::{
    entity::tags::{AliasSet, Tag, TagDepth, TagId},
    error::ErrorKind,
    repository::tags::TagsRepository,
};
use chrono::{TimeZone, Utc};
use postgres::tags::PostgresTagsRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        Some("ごらく部".to_string()),
        Some("ごらくぶ".to_string()),
        ["七森中☆ごらく部".to_string()].into_iter(),
        [].into_iter(),
        TagDepth::new(2, 2),
    ).await.unwrap();

    assert_eq!(actual.name, "ごらく部".to_string());
    assert_eq!(actual.kana, "ごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["七森中☆ごらく部".to_string()])));
    assert_eq!(actual.parent, Some(Box::new(Tag {
        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
        name: "ゆるゆり".to_string(),
        kana: "ゆるゆり".to_string(),
        aliases: AliasSet::default(),
        parent: None,
        children: Vec::new(),
        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
    })));
    assert_eq!(actual.children, vec![
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
    ]);
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("name"), "ごらく部");
    assert_eq!(actual.get::<&str, &str>("kana"), "ごらくぶ");
    assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["七森中☆ごらく部".to_string()]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn without_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        Some("ごらく部".to_string()),
        Some("ごらくぶ".to_string()),
        ["七森中☆ごらく部".to_string()].into_iter(),
        [].into_iter(),
        TagDepth::new(0, 0),
    ).await.unwrap();

    assert_eq!(actual.name, "ごらく部".to_string());
    assert_eq!(actual.kana, "ごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["七森中☆ごらく部".to_string()])));
    assert_eq!(actual.parent, None);
    assert_eq!(actual.children, Vec::new());
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("name"), "ごらく部");
    assert_eq!(actual.get::<&str, &str>("kana"), "ごらくぶ");
    assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["七森中☆ごらく部".to_string()]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn root_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
        None,
        None,
        [].into_iter(),
        [].into_iter(),
        TagDepth::new(0, 0),
    ).await.unwrap_err();

assert_matches!(actual.kind(), ErrorKind::TagUpdatingRoot);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        None,
        [].into_iter(),
        [].into_iter(),
        TagDepth::new(0, 0),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagNotFound { id } if id == &TagId::from(uuid!("11111111-1111-1111-1111-111111111111")));
}
