use std::hash::{Hash, Hasher};

use derive_more::{Deref, Display, From};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Display, Eq, From, Ord, PartialEq, PartialOrd)]
pub struct TagTypeId(Uuid);

#[derive(Clone, Debug, Eq, Ord, PartialOrd)]
pub struct TagType {
    pub id: TagTypeId,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Error)]
pub enum TagTypeError {
    #[error("tag type not found: {0}")]
    NotFound(TagTypeId),
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use compiled_uuid::uuid;

    use super::*;

    #[test]
    fn tag_type_hash_equals_by_id() {
        let a = TagType {
            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            slug: "foo".to_string(),
            name: "foo".to_string(),
        };

        let b = TagType {
            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            slug: "bar".to_string(),
            name: "bar".to_string(),
        };

        let mut hash_set = HashSet::new();
        hash_set.insert(a);

        let actual = hash_set.contains(&b);

        assert!(actual);
    }

    #[test]
    fn tag_type_hash_not_equals_by_id() {
        let a = TagType {
            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            slug: "foo".to_string(),
            name: "foo".to_string(),
        };

        let b = TagType {
            id: TagTypeId::from(uuid!("55555555-5555-5555-5555-555555555555")),
            slug: "foo".to_string(),
            name: "foo".to_string(),
        };

        let mut hash_set = HashSet::new();
        hash_set.insert(a);

        let actual = hash_set.contains(&b);

        assert!(!actual);
    }
}
