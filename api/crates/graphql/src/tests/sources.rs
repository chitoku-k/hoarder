use domain::entity::external_services;
use pretty_assertions::assert_eq;
use serde_json::json;

use crate::sources::{
    ExternalMetadata,
    ExternalMetadataId,
    ExternalMetadataIdCreatorId,
    ExternalMetadataIdOptionalCreatorId,
    ExternalMetadataUrl,
};

#[test]
fn convert_bluesky() {
    let metadata = ExternalMetadata::Bluesky(ExternalMetadataIdCreatorId { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() });

    let metadata = external_services::ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Bluesky(ExternalMetadataIdCreatorId { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() }));
}

#[test]
fn convert_fantia() {
    let metadata = ExternalMetadata::Fantia(ExternalMetadataId { id: "123456789".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Fantia { id: 123456789 });

    let metadata = external_services::ExternalMetadata::Fantia { id: 123456789 };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Fantia(ExternalMetadataId { id: "123456789".to_string() }));
}

#[test]
fn convert_mastodon() {
    let metadata = ExternalMetadata::Mastodon(ExternalMetadataIdCreatorId { id: "123456789".to_string(), creator_id: "creator_01".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() });

    let metadata = external_services::ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Mastodon(ExternalMetadataIdCreatorId { id: "123456789".to_string(), creator_id: "creator_01".to_string() }));
}

#[test]
fn convert_misskey() {
    let metadata = ExternalMetadata::Misskey(ExternalMetadataId { id: "abcdefghi".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Misskey { id: "abcdefghi".to_string() });

    let metadata = external_services::ExternalMetadata::Misskey { id: "abcdefghi".to_string() };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Misskey(ExternalMetadataId { id: "abcdefghi".to_string() }));
}

#[test]
fn convert_nijie() {
    let metadata = ExternalMetadata::Nijie(ExternalMetadataId { id: "123456789".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Nijie { id: 123456789 });

    let metadata = external_services::ExternalMetadata::Nijie { id: 123456789 };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Nijie(ExternalMetadataId { id: "123456789".to_string() }));
}

#[test]
fn convert_pixiv() {
    let metadata = ExternalMetadata::Pixiv(ExternalMetadataId { id: "123456789".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Pixiv { id: 123456789 });

    let metadata = external_services::ExternalMetadata::Pixiv { id: 123456789 };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Pixiv(ExternalMetadataId { id: "123456789".to_string() }));
}

#[test]
fn convert_pixiv_fanbox() {
    let metadata = ExternalMetadata::PixivFanbox(ExternalMetadataIdCreatorId { id: "123456789".to_string(), creator_id: "creator_01".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() });

    let metadata = external_services::ExternalMetadata::PixivFanbox { id: 123456789, creator_id: "creator_01".to_string() };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::PixivFanbox(ExternalMetadataIdCreatorId { id: "123456789".to_string(), creator_id: "creator_01".to_string() }));
}

#[test]
fn convert_pleroma() {
    let metadata = ExternalMetadata::Pleroma(ExternalMetadataId { id: "abcdefghi".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Pleroma { id: "abcdefghi".to_string() });

    let metadata = external_services::ExternalMetadata::Pleroma { id: "abcdefghi".to_string() };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Pleroma(ExternalMetadataId { id: "abcdefghi".to_string() }));
}

#[test]
fn convert_seiga() {
    let metadata = ExternalMetadata::Seiga(ExternalMetadataId { id: "123456789".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Seiga { id: 123456789 });

    let metadata = external_services::ExternalMetadata::Seiga { id: 123456789 };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Seiga(ExternalMetadataId { id: "123456789".to_string() }));
}

#[test]
fn convert_skeb() {
    let metadata = ExternalMetadata::Skeb(ExternalMetadataIdCreatorId { id: "123456789".to_string(), creator_id: "creator_01".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Skeb { id: 123456789, creator_id: "creator_01".to_string() });

    let metadata = external_services::ExternalMetadata::Skeb { id: 123456789, creator_id: "creator_01".to_string() };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Skeb(ExternalMetadataIdCreatorId { id: "123456789".to_string(), creator_id: "creator_01".to_string() }));
}

#[test]
fn convert_threads() {
    let metadata = ExternalMetadata::Threads(ExternalMetadataIdOptionalCreatorId { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) });

    let metadata = external_services::ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Threads(ExternalMetadataIdOptionalCreatorId { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) }));
}

#[test]
fn convert_website() {
    let metadata = ExternalMetadata::Website(ExternalMetadataUrl { url: "https://example.com".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Website { url: "https://example.com".to_string() });

    let metadata = external_services::ExternalMetadata::Website { url: "https://example.com".to_string() };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Website(ExternalMetadataUrl { url: "https://example.com".to_string() }));
}

#[test]
fn convert_x() {
    let metadata = ExternalMetadata::X(ExternalMetadataIdOptionalCreatorId { id: "123456789".to_string(), creator_id: Some("creator_01".to_string()) });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::X { id: 123456789, creator_id: Some("creator_01".to_string()) });

    let metadata = external_services::ExternalMetadata::X { id: 123456789, creator_id: Some("creator_01".to_string()) };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::X(ExternalMetadataIdOptionalCreatorId { id: "123456789".to_string(), creator_id: Some("creator_01".to_string()) }));
}

#[test]
fn convert_xfolio() {
    let metadata = ExternalMetadata::Xfolio(ExternalMetadataIdCreatorId { id: "123456789".to_string(), creator_id: "creator_01".to_string() });
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() });

    let metadata = external_services::ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() };
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Xfolio(ExternalMetadataIdCreatorId { id: "123456789".to_string(), creator_id: "creator_01".to_string() }));
}

#[test]
fn convert_custom() {
    let metadata = ExternalMetadata::Custom(json!({ "id": 123456789 }));
    let actual = external_services::ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, external_services::ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string()));

    let metadata = external_services::ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string());
    let actual = ExternalMetadata::try_from(metadata).unwrap();

    assert_eq!(actual, ExternalMetadata::Custom(json!({ "id": 123456789 })));
}
