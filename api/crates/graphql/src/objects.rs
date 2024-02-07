use async_graphql::{Enum, SimpleObject};
use domain::entity::objects;

#[derive(SimpleObject)]
pub(crate) struct ObjectEntry {
    name: String,
    path: String,
    kind: ObjectKind,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ObjectKind {
   Container,
   Object,
   Unknown,
}

impl From<objects::Entry> for ObjectEntry {
    fn from(entry: objects::Entry) -> Self {
        Self {
            name: entry.name,
            path: entry.path,
            kind: entry.kind.into(),
        }
    }
}

impl From<objects::Kind> for ObjectKind {
    fn from(kind: objects::Kind) -> Self {
        use objects::Kind::*;
        match kind {
            Container => ObjectKind::Container,
            Object => ObjectKind::Object,
            Unknown => ObjectKind::Unknown,
        }
    }
}

impl From<ObjectKind> for objects::Kind {
    fn from(kind: ObjectKind) -> Self {
        use objects::Kind::*;
        match kind {
            ObjectKind::Container => Container,
            ObjectKind::Object => Object,
            ObjectKind::Unknown => Unknown,
        }
    }
}
