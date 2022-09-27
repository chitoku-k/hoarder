use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use derive_more::{Deref, Display, From};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::entity::{
    replicas::Replica,
    sources::Source,
    tag_types::TagType,
    tags::Tag,
};

#[derive(Clone, Copy, Debug, Default, Deref, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd)]
pub struct MediumId(Uuid);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Medium {
    pub id: MediumId,
    pub sources: Vec<Source>,
    pub tags: BTreeMap<TagType, Vec<Tag>>,
    pub replicas: Vec<Replica>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Error)]
pub enum MediumError {
    #[error("medium not found: {0}")]
    NotFound(MediumId),
}

impl Default for Medium {
    fn default() -> Self {
        Self {
            id: Default::default(),
            sources: Default::default(),
            tags: Default::default(),
            replicas: Default::default(),
            created_at: NaiveDateTime::MIN,
            updated_at: NaiveDateTime::MIN,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn medium_default() {
        let actual = Medium::default();

        assert_eq!(actual, Medium {
            id: MediumId::from(Uuid::nil()),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDateTime::MIN,
            updated_at: NaiveDateTime::MIN,
        });
    }
}
