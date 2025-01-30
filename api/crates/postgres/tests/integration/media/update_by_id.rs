use std::collections::BTreeSet;

use chrono::{DateTime, TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        media::MediumId,
        replicas::{Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{AliasSet, Tag, TagDepth, TagId},
    },
    error::ErrorKind,
    repository::media::MediaRepository,
};
use futures::TryStreamExt;
use ordermap::OrderMap;
use postgres::media::PostgresMediaRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [].into_iter(),
        None,
        None,
        false,
        false,
    ).await.unwrap();

    assert_eq!(actual.sources, Vec::new());
    assert_eq!(actual.tags, OrderMap::<TagType, Vec<Tag>>::new());
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

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
async fn with_tags_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [].into_iter(),
        None,
        Some(TagDepth::new(2, 2)),
        false,
        false,
    ).await.unwrap();

    assert_eq!(actual.sources, Vec::new());
    assert_eq!(actual.tags, {
        let mut tags = OrderMap::new();
        tags.insert(
            TagType {
                id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                slug: "character".to_string(),
                name: "キャラクター".to_string(),
                kana: "キャラクター".to_string(),
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
        tags.insert(
            TagType {
                id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                slug: "work".to_string(),
                name: "作品".to_string(),
                kana: "さくひん".to_string(),
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
        tags
    });
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

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
async fn with_replicas_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [].into_iter(),
        None,
        None,
        true,
        false,
    ).await.unwrap();

    assert_eq!(actual.sources, Vec::new());
    assert_eq!(actual.tags, OrderMap::<TagType, Vec<Tag>>::new());
    assert_eq!(actual.replicas, vec![
        Replica {
            id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            display_order: 1,
            thumbnail: Some(Thumbnail {
                id: ThumbnailId::from(uuid!("9785df5f-f975-4253-9b50-b5e3abb92a70")),
                size: Size::new(1, 1),
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
            }),
            original_url: "file:///1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
            mime_type: Some("image/png".to_string()),
            size: Some(Size::new(1920, 1600)),
            status: ReplicaStatus::Ready,
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            display_order: 2,
            thumbnail: Some(Thumbnail {
                id: ThumbnailId::from(uuid!("41512f05-a89e-4d2f-899b-9bf7b201679e")),
                size: Size::new(1, 1),
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 12).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
            }),
            original_url: "file:///6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
            mime_type: Some("image/png".to_string()),
            size: Some(Size::new(1920, 1600)),
            status: ReplicaStatus::Ready,
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            display_order: 3,
            thumbnail: None,
            original_url: "file:///12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
            mime_type: Some("image/png".to_string()),
            size: Some(Size::new(1920, 1600)),
            status: ReplicaStatus::Ready,
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
    ]);
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

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
async fn with_sources_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [].into_iter(),
        None,
        None,
        false,
        true,
    ).await.unwrap();

    assert_eq!(actual.sources, vec![
        Source {
            id: SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                slug: "skeb".to_string(),
                kind: "skeb".to_string(),
                name: "Skeb".to_string(),
                base_url: Some("https://skeb.jp".to_string()),
                url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
            },
            external_metadata: ExternalMetadata::Skeb { id: 1111, creator_id: "creator_02".to_string() },
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 13).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                slug: "x".to_string(),
                kind: "x".to_string(),
                name: "X".to_string(),
                base_url: Some("https://x.com".to_string()),
                url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
            },
            external_metadata: ExternalMetadata::X { id: 333333333333, creator_id: Some("creator_03".to_string()) },
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
        },
    ]);
    assert_eq!(actual.tags, OrderMap::<TagType, Vec<Tag>>::new());
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

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
async fn reorder_replicas_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap()),
        None,
        false,
        false,
    ).await.unwrap();

    assert_eq!(actual.sources, Vec::new());
    assert_eq!(actual.tags, OrderMap::<TagType, Vec<Tag>>::new());
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

    let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap());

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
async fn reorder_replicas_with_tags_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap()),
        Some(TagDepth::new(2, 2)),
        false,
        false,
    ).await.unwrap();

    assert_eq!(actual.sources, Vec::new());
    assert_eq!(actual.tags, {
        let mut tags = OrderMap::new();
        tags.insert(
            TagType {
                id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
                slug: "character".to_string(),
                name: "キャラクター".to_string(),
                kana: "キャラクター".to_string(),
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
        tags.insert(
            TagType {
                id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
                slug: "work".to_string(),
                name: "作品".to_string(),
                kana: "さくひん".to_string(),
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
        tags
    });
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

    let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap());

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
async fn reorder_replicas_with_replicas_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap()),
        None,
        true,
        false,
    ).await.unwrap();

    assert_eq!(actual.sources, Vec::new());
    assert_eq!(actual.tags, OrderMap::<TagType, Vec<Tag>>::new());
    assert_eq!(actual.replicas, vec![
        Replica {
            id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            display_order: 1,
            thumbnail: Some(Thumbnail {
                id: ThumbnailId::from(uuid!("41512f05-a89e-4d2f-899b-9bf7b201679e")),
                size: Size::new(1, 1),
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 12).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
            }),
            original_url: "file:///6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
            mime_type: Some("image/png".to_string()),
            size: Some(Size::new(1920, 1600)),
            status: ReplicaStatus::Ready,
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            display_order: 2,
            thumbnail: None,
            original_url: "file:///12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
            mime_type: Some("image/png".to_string()),
            size: Some(Size::new(1920, 1600)),
            status: ReplicaStatus::Ready,
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            display_order: 3,
            thumbnail: Some(Thumbnail {
                id: ThumbnailId::from(uuid!("9785df5f-f975-4253-9b50-b5e3abb92a70")),
                size: Size::new(1, 1),
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
            }),
            original_url: "file:///1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
            mime_type: Some("image/png".to_string()),
            size: Some(Size::new(1920, 1600)),
            status: ReplicaStatus::Ready,
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
    ]);
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

    let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap());

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
async fn reorder_replicas_with_sources_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap()),
        None,
        false,
        true,
    ).await.unwrap();

    assert_eq!(actual.sources, vec![
        Source {
            id: SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                slug: "skeb".to_string(),
                kind: "skeb".to_string(),
                name: "Skeb".to_string(),
                base_url: Some("https://skeb.jp".to_string()),
                url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
            },
            external_metadata: ExternalMetadata::Skeb { id: 1111, creator_id: "creator_02".to_string() },
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 13).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                slug: "x".to_string(),
                kind: "x".to_string(),
                name: "X".to_string(),
                base_url: Some("https://x.com".to_string()),
                url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
            },
            external_metadata: ExternalMetadata::X { id: 333333333333, creator_id: Some("creator_03".to_string()) },
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
        },
    ]);
    assert_eq!(actual.tags, OrderMap::<TagType, Vec<Tag>>::new());
    assert_eq!(actual.replicas, Vec::new());
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

    let actual = sqlx::query(r#"SELECT "created_at" FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap());

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
async fn reorder_too_few_replicas_fails(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap()),
        None,
        false,
        false,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::MediumReplicasNotMatch { medium_id, expected_replicas, actual_replicas } if (medium_id, expected_replicas, actual_replicas) == (
        &MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        &vec![
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
        ],
        &vec![
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
        ],
    ));
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn reorder_too_many_replicas_fails(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap()),
        None,
        false,
        false,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::MediumReplicasNotMatch { medium_id, expected_replicas, actual_replicas } if (medium_id, expected_replicas, actual_replicas) == (
        &MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        &vec![
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
        ],
        &vec![
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
        ],
    ));
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn reorder_replicas_mismatch_fails(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        [
            SourceId::from(uuid!("6807b3f6-6325-4212-bba5-bdb48150bb69")),
            SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
        ].into_iter(),
        [
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            ),
            (
                TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 4, 5, 6, 7, 8).unwrap()),
        None,
        false,
        false,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::MediumReplicasNotMatch { medium_id, expected_replicas, actual_replicas } if (medium_id, expected_replicas, actual_replicas) == (
        &MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        &vec![
            ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
        ],
        &vec![
            ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
        ],
    ));
}
