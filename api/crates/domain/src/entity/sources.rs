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
        if self.external_metadata.kind().is_none_or(|kind| kind == self.external_service.kind) {
            Ok(())
        } else {
            Err(ErrorKind::SourceMetadataNotMatch { kind: self.external_service.kind.clone() })?
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
}
