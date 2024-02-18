use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use domain::entity::objects;
use serde::Serialize;

#[derive(Debug, Serialize, SimpleObject)]
pub(crate) struct ObjectEntry {
    name: String,
    path: String,
    url: String,
    kind: ObjectKind,
    metadata: Option<ObjectEntryMetadata>,
}

#[derive(Debug, Serialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectEntryMetadata {
    size: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    accessed_at: DateTime<Utc>,
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum ObjectKind {
   Container,
   Object,
   Unknown,
}

impl From<objects::Entry> for ObjectEntry {
    fn from(entry: objects::Entry) -> Self {
        Self {
            name: entry.name,
            path: entry.path.into_inner(),
            url: entry.url.into_inner(),
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
