use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId},
    },
    repository::media::MediaRepository,
};
use ordermap::OrderMap;
use postgres::media::PostgresMediaRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids(
        [
            MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        ].into_iter(),
        None,
        true,
        false,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                    mime_type: Some("image/jpeg".to_string()),
                    size: Some(Size::new(1800, 2400)),
                    status: ReplicaStatus::Ready,
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("790dc278-2c53-4988-883c-43a037664b24")),
                    display_order: 2,
                    thumbnail: None,
                    original_url: "file:///790dc278-2c53-4988-883c-43a037664b24.jpg".to_string(),
                    mime_type: Some("image/jpeg".to_string()),
                    size: Some(Size::new(1800, 2400)),
                    status: ReplicaStatus::Ready,
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: OrderMap::new(),
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
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
    ]);
}
