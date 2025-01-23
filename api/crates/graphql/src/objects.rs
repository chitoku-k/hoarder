use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use domain::entity::objects;
use serde::Serialize;

/// An object entry represents an object in the storage.
#[derive(Debug, Serialize, SimpleObject)]
pub(crate) struct ObjectEntry {
    /// The name of the object.
    name: String,
    /// The internal URL of the object.
    url: Option<String>,
    /// The kind of the object.
    kind: ObjectKind,
    /// The metadata of the object.
    metadata: Option<ObjectEntryMetadata>,
}

/// An object entry metadata represents attributes of an object in the storage.
#[derive(Debug, Serialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectEntryMetadata {
    /// The size of the object in bytes.
    size: u64,
    /// The date at which the object was created.
    created_at: Option<DateTime<Utc>>,
    /// The date at which the object was updated.
    updated_at: Option<DateTime<Utc>>,
    /// The date at which the object was accessed.
    accessed_at: Option<DateTime<Utc>>,
}

/// An object kind represents the kind of an object in the storage.
#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum ObjectKind {
   /// Container, e.g., a directory.
   Container,
   /// Object, e.g., a file.
   Object,
   /// Unknown.
   Unknown,
}

impl From<objects::Entry> for ObjectEntry {
    fn from(entry: objects::Entry) -> Self {
        Self {
            name: entry.name,
            url: entry.url.map(|u| u.into_inner()),
            kind: entry.kind.into(),
            metadata: entry.metadata.map(Into::into),
        }
    }
}

impl From<objects::EntryMetadata> for ObjectEntryMetadata {
    fn from(metadata: objects::EntryMetadata) -> Self {
        Self {
            size: metadata.size,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            accessed_at: metadata.accessed_at,
        }
    }
}

impl From<objects::EntryKind> for ObjectKind {
    fn from(kind: objects::EntryKind) -> Self {
        use objects::EntryKind::*;
        match kind {
            Container => ObjectKind::Container,
            Object => ObjectKind::Object,
            Unknown => ObjectKind::Unknown,
        }
    }
}

impl From<ObjectKind> for objects::EntryKind {
    fn from(kind: ObjectKind) -> Self {
        use objects::EntryKind::*;
        match kind {
            ObjectKind::Container => Container,
            ObjectKind::Object => Object,
            ObjectKind::Unknown => Unknown,
        }
    }
}
