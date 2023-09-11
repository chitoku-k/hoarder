use chrono::{DateTime, Utc};
use derive_more::{Constructor, Deref, Display, From};
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, PartialEq)]
pub struct ReplicaId(Uuid);

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, PartialEq)]
pub struct ThumbnailId(Uuid);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replica {
    pub id: ReplicaId,
    pub display_order: u32,
    pub thumbnail: Option<Thumbnail>,
    pub original_url: String,
    pub mime_type: String,
    pub size: Size,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Thumbnail {
    pub id: ThumbnailId,
    pub size: Size,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub struct ThumbnailImage {
    pub body: Vec<u8>,
    pub size: Size,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub struct OriginalImage {
    pub mime_type: &'static str,
    pub size: Size,
}

#[derive(Clone, Constructor, Copy, Debug, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Error)]
pub enum ReplicaError {
    #[error("replica not found: {0}")]
    NotFoundById(ReplicaId),
    #[error("replica not found by URL: {0}")]
    NotFoundByURL(String),
}

#[derive(Debug, Error)]
pub enum ThumbnailError {
    #[error("thumbnail not found: {0}")]
    NotFoundById(ThumbnailId),
}
