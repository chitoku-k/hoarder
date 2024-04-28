use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{entity::external_services::{ExternalMetadata, ExternalService}, error::{ErrorKind, Result}};

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SourceId(Uuid);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    pub id: SourceId,
    pub external_service: ExternalService,
    pub external_metadata: ExternalMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("source not found: {0}")]
    NotFound(SourceId),
}

impl Source {
    pub fn validate(&self) -> Result<()> {
        match (self.external_service.kind.as_str(), &self.external_metadata) {
            ("bluesky", &ExternalMetadata::Bluesky { .. }) => Ok(()),
            ("fantia", &ExternalMetadata::Fantia { .. }) => Ok(()),
            ("mastodon", &ExternalMetadata::Mastodon { .. }) => Ok(()),
            ("misskey", &ExternalMetadata::Misskey { .. }) => Ok(()),
            ("nijie", &ExternalMetadata::Nijie { .. }) => Ok(()),
            ("pixiv", &ExternalMetadata::Pixiv { .. }) => Ok(()),
            ("pixiv_fanbox", &ExternalMetadata::PixivFanbox { .. }) => Ok(()),
            ("pleroma", &ExternalMetadata::Pleroma { .. }) => Ok(()),
            ("seiga", &ExternalMetadata::Seiga { .. }) => Ok(()),
            ("skeb", &ExternalMetadata::Skeb { .. }) => Ok(()),
            ("threads", &ExternalMetadata::Threads { .. }) => Ok(()),
            ("twitter", &ExternalMetadata::Twitter { .. }) => Ok(()),
            ("website", &ExternalMetadata::Website { .. }) => Ok(()),
            (_, &ExternalMetadata::Custom(_)) => Ok(()),
            (kind, _) => Err(ErrorKind::SourceMetadataNotMatch { kind: kind.to_string() })?,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::external_services::ExternalServiceId;

    use chrono::TimeZone;
    use pretty_assertions::assert_matches;
    use uuid::uuid;

    use super::*;

    #[test]
    fn validate_succeeds_with_bluesky() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "bsky_social".to_string(),
                kind: "bluesky".to_string(),
                name: "bsky.social".to_string(),
                base_url: Some("https://bsky.social".to_string()),
            },
            external_metadata: ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() },
            created_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 6).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_fantia() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "fantia".to_string(),
                kind: "fantia".to_string(),
                name: "Fantia".to_string(),
                base_url: Some("https://fantia.jp".to_string()),
            },
            external_metadata: ExternalMetadata::Fantia { id: 1305295 },
            created_at: Utc.with_ymd_and_hms(2022, 6, 4, 19, 34, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 4, 19, 34, 0).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_mastodon() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "mastodon_social".to_string(),
                kind: "mastodon".to_string(),
                name: "mastodon.social".to_string(),
                base_url: Some("https://mastodon.social".to_string()),
            },
            external_metadata: ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() },
            created_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 6).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_misskey() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "misskey_io".to_string(),
                kind: "misskey".to_string(),
                name: "Misskey.io".to_string(),
                base_url: Some("https://misskey.io".to_string()),
            },
            external_metadata: ExternalMetadata::Misskey { id: "abcdefghi".to_string() },
            created_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 6).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_nijie() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "nijie".to_string(),
                kind: "nijie".to_string(),
                name: "ニジエ".to_string(),
                base_url: Some("https://nijie.info".to_string()),
            },
            external_metadata: ExternalMetadata::Nijie { id: 323512 },
            created_at: Utc.with_ymd_and_hms(2019, 7, 19, 18, 9, 54).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2019, 7, 19, 18, 9, 54).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_pixiv() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "pixiv".to_string(),
                kind: "pixiv".to_string(),
                name: "pixiv".to_string(),
                base_url: Some("https://www.pixiv.net".to_string()),
            },
            external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
            created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_pixiv_fanbox() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "pixiv_fanbox".to_string(),
                kind: "pixiv_fanbox".to_string(),
                name: "pixivFANBOX".to_string(),
                base_url: None,
            },
            external_metadata: ExternalMetadata::PixivFanbox { id: 178080, creator_id: "fairyeye".to_string() },
            created_at: Utc.with_ymd_and_hms(2018, 10, 18, 12, 22, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2018, 10, 18, 12, 22, 1).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_pleroma() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "udongein".to_string(),
                kind: "pleroma".to_string(),
                name: "Udongein".to_string(),
                base_url: Some("https://udongein.xyz".to_string()),
            },
            external_metadata: ExternalMetadata::Pleroma { id: "abcdefghi".to_string() },
            created_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 6).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_seiga() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "seiga".to_string(),
                kind: "seiga".to_string(),
                name: "ニコニコ静画".to_string(),
                base_url: Some("https://seiga.nicovideo.jp".to_string()),
            },
            external_metadata: ExternalMetadata::Seiga { id: 6452903 },
            created_at: Utc.with_ymd_and_hms(2017, 2, 1, 23, 34, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2017, 2, 1, 23, 34, 1).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_skeb() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "skeb".to_string(),
                kind: "skeb".to_string(),
                name: "Skeb".to_string(),
                base_url: Some("https://skeb.jp".to_string()),
            },
            external_metadata: ExternalMetadata::Skeb { id: 18, creator_id: "pieleaf_x2".to_string() },
            created_at: Utc.with_ymd_and_hms(2021, 7, 22, 20, 40, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2021, 7, 22, 20, 40, 1).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_threads() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "threads".to_string(),
                kind: "threads".to_string(),
                name: "Threads".to_string(),
                base_url: Some("https://www.threads.net".to_string()),
            },
            external_metadata: ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) },
            created_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 6).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_twitter() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "twitter".to_string(),
                kind: "twitter".to_string(),
                name: "Twitter".to_string(),
                base_url: Some("https://twitter.com".to_string()),
            },
            external_metadata: ExternalMetadata::Twitter { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
            created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
        };

        let actual = source.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn validate_succeeds_with_website() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "website".to_string(),
                kind: "website".to_string(),
                name: "Website".to_string(),
                base_url: None,
            },
            external_metadata: ExternalMetadata::Website { url: "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885".to_string() },
            created_at: Utc.with_ymd_and_hms(2022, 4, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 4, 1, 0, 0, 1).unwrap(),
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
            },
            external_metadata: ExternalMetadata::Fantia { id: 1305295 },
            created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        };

        let actual = source.validate().unwrap_err();
        assert_matches!(actual.kind(), ErrorKind::SourceMetadataNotMatch { kind } if kind == "website");
    }
}
