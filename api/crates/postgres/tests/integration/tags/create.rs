use std::collections::BTreeSet;

use domain::{
    entity::tags::{AliasSet, Tag, TagDepth, TagId},
    repository::tags::TagsRepository,
};
use chrono::{TimeZone, Utc};
use futures::TryStreamExt;
use postgres::tags::PostgresTagsRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_parent_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.create(
        "七森中☆生徒会",
        "ななもりちゅうせいとかい",
        ["生徒会".to_string(), "七森中生徒会".to_string()].into_iter(),
        Some(TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))),
        TagDepth::new(2, 2),
    ).await.unwrap();

    let actual_id = actual.id;
    assert_eq!(actual.name, "七森中☆生徒会".to_string());
    assert_eq!(actual.kana, "ななもりちゅうせいとかい");
    assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["生徒会".to_string(), "七森中生徒会".to_string()])));
    assert_eq!(
        actual.parent,
        Some(Box::new(Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        })),
    );
    assert_eq!(actual.children, Vec::new());

    let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("name"), "七森中☆生徒会");
    assert_eq!(actual.get::<&str, &str>("kana"), "ななもりちゅうせいとかい");
    assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["生徒会".to_string(), "七森中生徒会".to_string()]);

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" = $1 ORDER BY "distance" DESC"#)
        .bind(*actual_id)
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 3);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[0].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[1].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), *actual_id);
    assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[2].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn without_parent_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.create(
        "七森中☆生徒会",
        "ななもりちゅうせいとかい",
        ["生徒会".to_string(), "七森中生徒会".to_string()].into_iter(),
        None,
        TagDepth::new(2, 2),
    ).await.unwrap();

    let actual_id = actual.id;
    assert_eq!(actual.name, "七森中☆生徒会".to_string());
    assert_eq!(actual.kana, "ななもりちゅうせいとかい");
    assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["生徒会".to_string(), "七森中生徒会".to_string()])));
    assert_eq!(actual.parent, None);
    assert_eq!(actual.children, Vec::new());

    let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("name"), "七森中☆生徒会");
    assert_eq!(actual.get::<&str, &str>("kana"), "ななもりちゅうせいとかい");
    assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["生徒会".to_string(), "七森中生徒会".to_string()]);

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" = $1 ORDER BY "distance" DESC"#)
        .bind(*actual_id)
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 2);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[0].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), *actual_id);
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[1].get::<i32, &str>("distance"), 0);
}
