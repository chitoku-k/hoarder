use std::hash::{Hash, Hasher};

use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TagTypeId(Uuid);

#[derive(Clone, Debug, Eq, Ord, PartialOrd)]
pub struct TagType {
    pub id: TagTypeId,
    pub slug: String,
    pub name: String,
    pub kana: String,
}

impl Hash for TagType {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl PartialEq for TagType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
