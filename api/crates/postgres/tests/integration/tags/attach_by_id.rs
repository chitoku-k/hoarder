use std::collections::BTreeSet;

use domain::{
    entity::tags::{AliasSet, Tag, TagDepth, TagId},
    error::ErrorKind,
    repository::tags::TagsRepository,
};
use chrono::{TimeZone, Utc};
use futures::TryStreamExt;
use postgres::tags::PostgresTagsRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(2, 2),
    ).await.unwrap();

    assert_eq!(actual.name, "七森中☆ごらく部".to_string());
    assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::default());
    assert_eq!(actual.parent, Some(Box::new(Tag {
        id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        name: "東方Project".to_string(),
        kana: "とうほうProject".to_string(),
        aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
        parent: None,
        children: Vec::new(),
        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
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
    assert_eq!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap());

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6, $7) ORDER BY "descendant_id", "distance" DESC"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 23);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[0].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[1].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[2].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[3].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[4].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[5].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[8].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[9].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[10].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[14].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[15].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[16].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[16].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[16].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[17].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[17].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[17].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[18].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[18].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[18].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[19].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[19].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[19].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[20].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[20].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[20].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[21].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[21].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[21].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[22].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[22].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[22].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn without_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(0, 0),
    ).await.unwrap();

    assert_eq!(actual.name, "七森中☆ごらく部".to_string());
    assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::default());
    assert_eq!(actual.parent, None);
    assert_eq!(actual.children, Vec::new());
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap());
    assert_eq!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap());

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6, $7) ORDER BY "descendant_id", "distance" DESC"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 23);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[0].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[1].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[2].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[3].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[4].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[5].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[8].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[9].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[10].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[14].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[15].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[16].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[16].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[16].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[17].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[17].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[17].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[18].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[18].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[18].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[19].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[19].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[19].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[20].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[20].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[20].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[21].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[21].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[21].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[22].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[22].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[22].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn root_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(0, 0),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagAttachingRoot);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn root_parent_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
        TagDepth::new(0, 0),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagAttachingRoot);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn self_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(0, 0),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagAttachingToItself { id } if id == &TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")));
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn descendant_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
        TagDepth::new(0, 0),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagAttachingToDescendant { id } if id == &TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")));
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn non_existing_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(0, 0),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagNotFound { id } if id == &TagId::from(uuid!("11111111-1111-1111-1111-111111111111")));
}
