use domain::entity::external_services::ExternalMetadata;
use pretty_assertions::assert_eq;
use serde_json::json;

use crate::sources::{PostgresExternalServiceMetadata, PostgresExternalServiceMetadataExtra, PostgresExternalServiceMetadataFull};

#[test]
fn convert_bluesky() {
    let metadata = PostgresExternalServiceMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::Bluesky {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() });

    let metadata = ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Bluesky {});
}

#[test]
fn convert_fantia() {
    let metadata = PostgresExternalServiceMetadata::Fantia { id: 123456789 };
    let extra = PostgresExternalServiceMetadataExtra::Fantia {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Fantia { id: 123456789 });

    let metadata = ExternalMetadata::Fantia { id: 123456789 };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Fantia { id: 123456789 });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Fantia {});
}

#[test]
fn convert_mastodon() {
    let metadata = PostgresExternalServiceMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::Mastodon {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() });

    let metadata = ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Mastodon {});
}

#[test]
fn convert_misskey() {
    let metadata = PostgresExternalServiceMetadata::Misskey { id: "abcdefghi".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::Misskey {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Misskey { id: "abcdefghi".to_string() });

    let metadata = ExternalMetadata::Misskey { id: "abcdefghi".to_string() };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Misskey { id: "abcdefghi".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Misskey {});
}

#[test]
fn convert_nijie() {
    let metadata = PostgresExternalServiceMetadata::Nijie { id: 123456789 };
    let extra = PostgresExternalServiceMetadataExtra::Nijie {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Nijie { id: 123456789 });

    let metadata = ExternalMetadata::Nijie { id: 123456789 };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Nijie { id: 123456789 });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Nijie {});
}

#[test]
fn convert_pixiv() {
    let metadata = PostgresExternalServiceMetadata::Pixiv { id: 123456789 };
    let extra = PostgresExternalServiceMetadataExtra::Pixiv {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Pixiv { id: 123456789 });

    let metadata = ExternalMetadata::Pixiv { id: 123456789 };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Pixiv { id: 123456789 });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Pixiv {});
}

#[test]
fn convert_pixiv_fanbox() {
    let metadata = PostgresExternalServiceMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::PixivFanbox {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() });

    let metadata = ExternalMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::PixivFanbox {});
}

#[test]
fn convert_pleroma() {
    let metadata = PostgresExternalServiceMetadata::Pleroma { id: "abcdefghi".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::Pleroma {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Pleroma { id: "abcdefghi".to_string() });

    let metadata = ExternalMetadata::Pleroma { id: "abcdefghi".to_string() };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Pleroma { id: "abcdefghi".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Pleroma {});
}

#[test]
fn convert_seiga() {
    let metadata = PostgresExternalServiceMetadata::Seiga { id: 123456789 };
    let extra = PostgresExternalServiceMetadataExtra::Seiga {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Seiga { id: 123456789 });

    let metadata = ExternalMetadata::Seiga { id: 123456789 };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Seiga { id: 123456789 });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Seiga {});
}

#[test]
fn convert_skeb() {
    let metadata = PostgresExternalServiceMetadata::Skeb { id: 123456789, creator_id: "creator_01".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::Skeb {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Skeb { id: 123456789, creator_id: "creator_01".to_string() });

    let metadata = ExternalMetadata::Skeb { id: 123456789, creator_id: "creator_01".to_string() };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Skeb { id: 123456789, creator_id: "creator_01".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Skeb {});
}

#[test]
fn convert_threads() {
    let metadata = PostgresExternalServiceMetadata::Threads { id: "abcdefghi".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::Threads { creator_id: Some("creator_01".to_string()) };
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) });

    let metadata = ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Threads { id: "abcdefghi".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Threads { creator_id: Some("creator_01".to_string()) });
}

#[test]
fn convert_website() {
    let metadata = PostgresExternalServiceMetadata::Website { url: "https://example.com".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::Website {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Website { url: "https://example.com".to_string() });

    let metadata = ExternalMetadata::Website { url: "https://example.com".to_string() };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Website { url: "https://example.com".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Website {});
}

#[test]
fn convert_x() {
    let metadata = PostgresExternalServiceMetadata::X { id: 123456789 };
    let extra = PostgresExternalServiceMetadataExtra::X { creator_id: Some("creator_01".to_string()) };
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::X { id: 123456789, creator_id: Some("creator_01".to_string()) });

    let metadata = ExternalMetadata::X { id: 123456789, creator_id: Some("creator_01".to_string()) };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::X { id: 123456789 });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::X { creator_id: Some("creator_01".to_string()) });
}

#[test]
fn convert_xfolio() {
    let metadata = PostgresExternalServiceMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() };
    let extra = PostgresExternalServiceMetadataExtra::Xfolio {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() });

    let metadata = ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() };
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() });
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Xfolio {});
}

#[test]
fn convert_custom() {
    let metadata = PostgresExternalServiceMetadata::Custom(json!({ "id": 123456789 }));
    let extra = PostgresExternalServiceMetadataExtra::Custom {};
    let actual = ExternalMetadata::try_from(PostgresExternalServiceMetadataFull(metadata, extra)).unwrap();

    assert_eq!(actual, ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string()));

    let metadata = ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string());
    let actual = PostgresExternalServiceMetadataFull::try_from(metadata).unwrap();

    assert_eq!(actual.0, PostgresExternalServiceMetadata::Custom(json!({ "id": 123456789 })));
    assert_eq!(actual.1, PostgresExternalServiceMetadataExtra::Custom {});
}
