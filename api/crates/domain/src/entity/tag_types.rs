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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use uuid::uuid;

    use super::*;

    #[test]
    fn tag_type_hash_equals_by_id() {
        let a = TagType {
            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            slug: "foo".to_string(),
            name: "foo".to_string(),
            kana: "foo".to_string(),
        };

        let b = TagType {
            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            slug: "bar".to_string(),
            name: "bar".to_string(),
            kana: "bar".to_string(),
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
            kana: "foo".to_string(),
        };

        let b = TagType {
            id: TagTypeId::from(uuid!("55555555-5555-5555-5555-555555555555")),
            slug: "foo".to_string(),
            name: "foo".to_string(),
            kana: "foo".to_string(),
        };

        let mut hash_set = HashSet::new();
        hash_set.insert(a);

        let actual = hash_set.contains(&b);

        assert!(!actual);
    }
}
