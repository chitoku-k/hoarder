use chrono::{TimeZone, Utc};
use pretty_assertions::assert_matches;
use uuid::uuid;

use crate::{entity::{external_services::{ExternalMetadata, ExternalService, ExternalServiceId}, sources::{Source, SourceId}}, error::ErrorKind};

#[test]
fn url_succeeds() {
    let source = Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
        },
        external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
    };

    let actual = source.url().unwrap();
    assert_eq!(actual, "https://x.com/_namori_/status/727620202049900544");
}

#[test]
fn validate_succeeds() {
    let source = Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
        },
        external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
    };

    let actual = source.validate();
    assert!(actual.is_ok());
}

#[test]
fn validate_succeeds_with_custom() {
    let source = Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "custom".to_string(),
            kind: "custom".to_string(),
            name: "Custom".to_string(),
            base_url: None,
            url_pattern: None,
        },
        external_metadata: ExternalMetadata::Custom(r#"{"id":42}"#.to_string()),
        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 1).unwrap(),
    };

    let actual = source.validate();
    assert!(actual.is_ok());
}

#[test]
fn validate_fails() {
    let source = Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "website".to_string(),
            kind: "website".to_string(),
            name: "Website".to_string(),
            base_url: None,
            url_pattern: None,
        },
        external_metadata: ExternalMetadata::Fantia { id: 1305295 },
        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
    };

    let actual = source.validate().unwrap_err();
    assert_matches!(actual.kind(), ErrorKind::SourceMetadataNotMatch { kind } if kind == "website");
}
