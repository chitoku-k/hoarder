use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, From};
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, PartialEq)]
pub struct ReplicaId(Uuid);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replica {
    pub id: ReplicaId,
    pub display_order: Option<u32>,
    pub has_thumbnail: bool,
    pub original_url: String,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplicaThumbnail {
    pub id: ReplicaId,
    pub display_order: Option<u32>,
    pub thumbnail: Option<Vec<u8>>,
    pub original_url: String,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum ReplicaError {
    #[error("replica not found: {0}")]
    NotFoundById(ReplicaId),
    #[error("replica not found by URL: {0}")]
    NotFoundByURL(String),
}
