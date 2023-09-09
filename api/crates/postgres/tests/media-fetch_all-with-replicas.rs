use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId, Size, Thumbnail, ThumbnailId},
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
        None,
        true,
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
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: 2,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: 2,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
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
        None,
        true,
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
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("69f9463e-9c29-48c9-a104-23341348ffec")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/69f9463e-9c29-48c9-a104-23341348ffec.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 17).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("040d009c-70df-4f55-ae55-df6e5fc57362")),
                    display_order: 2,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/040d009c-70df-4f55-ae55-df6e5fc57362.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 18).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 11).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("1524e043-a327-43ab-9a87-4e5ffa051cb7")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/1524e043-a327-43ab-9a87-4e5ffa051cb7.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 15).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("38505b5a-2e25-4325-8668-97cc39b57e73")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/38505b5a-2e25-4325-8668-97cc39b57e73.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 5).unwrap(),
                },
            ],
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
        None,
        true,
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
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/9b73469d-55fe-4017-aee8-dd8f8d7d067a.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                    display_order: 1,
                    thumbnail: Some(Thumbnail {
                        id: ThumbnailId::from(uuid!("9785df5f-f975-4253-9b50-b5e3abb92a70")),
                        size: Size::new(1, 1),
                        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                    }),
                    original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                    mime_type: "image/png".to_string(),
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
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: 3,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("fc874edd-6920-477d-a070-3c28203a070f")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/fc874edd-6920-477d-a070-3c28203a070f.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 12).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
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
        None,
        true,
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
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("38619a8c-68c7-4bdd-b7e4-169e51b1974e")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/38619a8c-68c7-4bdd-b7e4-169e51b1974e.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 13).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("e7f3765b-08ea-4217-a4da-4f56482c7d26")),
                    display_order: 2,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/e7f3765b-08ea-4217-a4da-4f56482c7d26.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 11).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("fc874edd-6920-477d-a070-3c28203a070f")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/fc874edd-6920-477d-a070-3c28203a070f.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 12).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                    display_order: 1,
                    thumbnail: Some(Thumbnail {
                        id: ThumbnailId::from(uuid!("9785df5f-f975-4253-9b50-b5e3abb92a70")),
                        size: Size::new(1, 1),
                        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                    }),
                    original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                    mime_type: "image/png".to_string(),
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
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: 3,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
            ],
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
        None,
        true,
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
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("91626dc4-3e2a-4028-8574-8feb3c817fd1")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: 2,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/7f0638e2-aa86-4b00-9e52-b0e803247a4b.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
            ],
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
        None,
        true,
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
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("fc874edd-6920-477d-a070-3c28203a070f")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/fc874edd-6920-477d-a070-3c28203a070f.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 12).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
                    display_order: 1,
                    thumbnail: Some(Thumbnail {
                        id: ThumbnailId::from(uuid!("9785df5f-f975-4253-9b50-b5e3abb92a70")),
                        size: Size::new(1, 1),
                        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                    }),
                    original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
                    mime_type: "image/png".to_string(),
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
                    original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
                    display_order: 3,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///var/lib/hoarder/9b73469d-55fe-4017-aee8-dd8f8d7d067a.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}
