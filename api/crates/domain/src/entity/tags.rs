use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use derive_more::{Constructor, Deref, Display, From};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TagId(Uuid);

#[derive(Clone, Constructor, Copy, Debug, Eq, From, PartialEq)]
pub struct TagDepth {
    parent: u32,
    children: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub kana: String,
    pub aliases: AliasSet,
    pub parent: Option<Box<Self>>,
    pub children: Vec<Self>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TagId {
    pub const fn root() -> Self {
        Self(Uuid::nil())
    }

    pub const fn is_root(&self) -> bool {
        self.0.is_nil()
    }
}

impl TagDepth {
    pub const fn parent(&self) -> u32 {
        self.parent
    }

    pub const fn children(&self) -> u32 {
        self.children
    }

    pub const fn has_parent(&self) -> bool {
        self.parent > 0
    }

    pub const fn has_children(&self) -> bool {
        self.children > 0
    }
}

#[derive(Clone, Constructor, Debug, Default, Deref, Eq, PartialEq)]
pub struct AliasSet(BTreeSet<String>);

impl AliasSet {
    pub fn add_all<T>(&mut self, aliases: T)
    where
        T: IntoIterator<Item = String>,
    {
        for alias in aliases {
            self.0.insert(alias);
        }
    }

    pub fn remove_all<T>(&mut self, aliases: T)
    where
        T: IntoIterator<Item = String>,
    {
        for alias in aliases {
            self.0.remove(&alias);
        }
    }

    pub fn into_inner(self) -> BTreeSet<String> {
        self.0
    }
}

impl From<Vec<String>> for AliasSet {
    fn from(v: Vec<String>) -> Self {
        Self::new(v.into_iter().collect())
    }
}

impl From<AliasSet> for Vec<String> {
    fn from(v: AliasSet) -> Self {
        v.0.into_iter().collect()
    }
}
