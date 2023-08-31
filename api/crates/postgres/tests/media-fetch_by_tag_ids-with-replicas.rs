use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId},
        tag_types::TagTypeId,
        tags::TagId,
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
                    display_order: Some(1),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
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
                    display_order: Some(1),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
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
        Order::Descending,
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
                    display_order: Some(1),
                    has_thumbnail: false,
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
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
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
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
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
async fn since_asc_succeeds(ctx: &DatabaseContext) {
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
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
        Order::Ascending,
        Direction::Forward,
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
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
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
                    display_order: Some(1),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: Some(2),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
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
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn since_desc_succeeds(ctx: &DatabaseContext) {
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
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
        Order::Descending,
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
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn until_asc_succeeds(ctx: &DatabaseContext) {
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
                    display_order: Some(1),
                    has_thumbnail: false,
                    original_url: "file:///var/lib/hoarder/91626dc4-3e2a-4028-8574-8feb3c817fd1.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("7f0638e2-aa86-4b00-9e52-b0e803247a4b")),
                    display_order: Some(2),
                    has_thumbnail: false,
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
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Descending,
        Direction::Backward,
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
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}
