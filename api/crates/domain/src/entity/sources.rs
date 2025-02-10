use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};
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

impl Source {
    pub fn url(&self) -> Option<String> {
        self.external_metadata.url(self.external_service.base_url.as_deref())
    }

    pub fn validate(&self) -> Result<()> {
        if self.external_metadata.kind().is_none_or(|kind| kind == self.external_service.kind) {
            Ok(())
        } else {
            Err(ErrorKind::SourceMetadataNotMatch { kind: self.external_service.kind.clone() })?
        }
    }
}
