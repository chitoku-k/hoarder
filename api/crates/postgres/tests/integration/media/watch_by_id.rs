use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId, ReplicaStatus, Size},
    },
    repository::media::MediaRepository,
};
use futures::{pin_mut, TryStreamExt};
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
    let stream = repository.watch_by_id(
        MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
        None,
        true,
        false,
    ).await.unwrap();
    pin_mut!(stream);

    let actual = stream.try_next().await.unwrap();

    assert_eq!(actual, Some(Medium {
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
    }));

    sqlx::query(r#"UPDATE "replicas" SET "mime_type" = $1, "width" = $2, "height" = $3, "phase" = $4 WHERE "id" = $5"#)
        .bind(None::<&str>)
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind("processing")
        .bind(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d"))
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query(r#"SELECT pg_notify($1, $2)"#)
        .bind("replicas")
        .bind(r#"{"id":"b7a54e0b-6ab3-4385-a18b-bacadff6b18d","medium_id":"2872ed9d-4db9-4b25-b86f-791ad009cc0a"}"#)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let actual = stream.try_next().await.unwrap();

    assert_eq!(actual, Some(Medium {
        id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
        sources: Vec::new(),
        tags: OrderMap::new(),
        replicas: vec![
            Replica {
                id: ReplicaId::from(uuid!("b7a54e0b-6ab3-4385-a18b-bacadff6b18d")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///b7a54e0b-6ab3-4385-a18b-bacadff6b18d.jpg".to_string(),
                mime_type: None,
                size: None,
                status: ReplicaStatus::Processing,
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
    }));
}
