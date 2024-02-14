use chrono::{DateTime, Utc};
use derive_more::{Constructor, Deref, Display, From};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, PartialEq)]
pub struct EntryPath(String);

#[derive(Clone, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, PartialEq)]
pub struct EntryUrl(String);

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub struct Entry {
    pub name: String,
    pub path: EntryPath,
    pub url: EntryUrl,
    pub kind: EntryKind,
    pub metadata: Option<EntryMetadata>,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub struct EntryMetadata {
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    Container,
    Object,
    Unknown,
}

impl EntryPath {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl EntryUrl {
    pub fn into_inner(self) -> String {
        self.0
    }
}
