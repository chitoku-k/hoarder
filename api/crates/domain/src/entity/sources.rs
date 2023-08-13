use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, From};
use thiserror::Error;
use uuid::Uuid;

use crate::entity::external_services::{ExternalMetadata, ExternalService, ExternalServiceError};

#[derive(Clone, Copy, Debug, Default, Deref, Display, Eq, From, PartialEq)]
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
    #[error("unknown external source")]
    UnknwonExternalSource,
}

impl Source {
    pub fn validate(&self) -> anyhow::Result<()> {
        match (self.external_service.slug.as_str(), &self.external_metadata) {
            ("fantia", &ExternalMetadata::Fantia { .. }) => Ok(()),
            ("nijie", &ExternalMetadata::Nijie { .. }) => Ok(()),
            ("pixiv", &ExternalMetadata::Pixiv { .. }) => Ok(()),
            ("pixiv_fanbox", &ExternalMetadata::PixivFanbox { .. }) => Ok(()),
            ("seiga", &ExternalMetadata::Seiga { .. }) => Ok(()),
            ("skeb", &ExternalMetadata::Skeb { .. }) => Ok(()),
            ("twitter", &ExternalMetadata::Twitter { .. }) => Ok(()),
            ("website", &ExternalMetadata::Website { .. }) => Ok(()),
            (_, &ExternalMetadata::Custom(_)) => Ok(()),
            (slug, _) => Err(ExternalServiceError::InvalidMetadata(slug.to_string()))?,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::external_services::ExternalServiceId;

    use chrono::TimeZone;
    use pretty_assertions::assert_eq;
    use uuid::uuid;

    use super::*;

    #[test]
    fn validate_succeeds_with_fantia() {
        let source = Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                slug: "fantia".to_string(),
                name: "Fantia".to_string(),
            },
            external_metadata: ExternalMetadata::Fantia { id: 1305295 },
            created_at: Utc.with_ymd_and_hms(2022, 6, 4, 19, 34, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 4, 19, 34, 0).unwrap(),
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
                name: "ニジエ".to_string(),
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
                name: "pixiv".to_string(),
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
                name: "pixivFANBOX".to_string(),
            },
            external_metadata: ExternalMetadata::PixivFanbox { id: 178080, creator_id: "fairyeye".to_string() },
            created_at: Utc.with_ymd_and_hms(2018, 10, 18, 12, 22, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2018, 10, 18, 12, 22, 1).unwrap(),
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
                name: "ニコニコ静画".to_string(),
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
                name: "Skeb".to_string(),
            },
            external_metadata: ExternalMetadata::Skeb { id: 18, creator_id: "pieleaf_x2".to_string() },
            created_at: Utc.with_ymd_and_hms(2021, 7, 22, 20, 40, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2021, 7, 22, 20, 40, 1).unwrap(),
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
                name: "Twitter".to_string(),
            },
            external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
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
                name: "Website".to_string(),
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
                name: "Custom".to_string(),
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
                name: "Website".to_string(),
            },
            external_metadata: ExternalMetadata::Fantia { id: 1305295 },
            created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        };

        let error = source.validate().unwrap_err();
        let actual: ExternalServiceError = error.downcast().unwrap();
        assert_eq!(actual, ExternalServiceError::InvalidMetadata("website".to_string()));
    }
}
