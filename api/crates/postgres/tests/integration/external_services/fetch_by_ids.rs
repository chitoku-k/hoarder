use domain::{
    entity::external_services::{ExternalService, ExternalServiceId},
    repository::external_services::ExternalServicesRepository,
};
use postgres::external_services::PostgresExternalServicesRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids([
        ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
    ].into_iter()).await.unwrap();

    assert_eq!(actual, vec![
        ExternalService {
            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            slug: "pixiv".to_string(),
            kind: "pixiv".to_string(),
            name: "pixiv".to_string(),
            base_url: Some("https://www.pixiv.net".to_string()),
            url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
        },
    ]);
}
