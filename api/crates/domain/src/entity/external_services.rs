use derive_more::{Deref, Display, From};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Display, Eq, From, PartialEq)]
pub struct ExternalServiceId(Uuid);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalService {
    pub id: ExternalServiceId,
    pub slug: String,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExternalMetadata {
    Fantia { id: u64 },
    Nijie { id: u64 },
    Pixiv { id: u64 },
    PixivFanbox { id: u64, creator_id: String },
    Seiga { id: u64 },
    Skeb { id: u64, creator_id: String },
    Twitter { id: u64 },
    Website { url: String },
    Custom(String),
}

#[derive(Debug, Eq, Error, PartialEq)]
pub enum ExternalServiceError {
    #[error("external service not found: {0}")]
    NotFound(ExternalServiceId),
    #[error("invalid metadata for {0}")]
    InvalidMetadata(String),
}
