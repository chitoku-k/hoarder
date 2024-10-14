use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, From};
use ordermap::OrderMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::entity::{
    replicas::Replica,
    sources::Source,
    tag_types::TagType,
    tags::Tag,
};

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct MediumId(Uuid);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Medium {
    pub id: MediumId,
    pub sources: Vec<Source>,
    pub tags: OrderMap<TagType, Vec<Tag>>,
    pub replicas: Vec<Replica>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum MediumError {
    #[error("medium not found: {0}")]
    NotFound(MediumId),
}
