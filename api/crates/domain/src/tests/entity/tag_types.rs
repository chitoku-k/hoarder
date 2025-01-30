use std::collections::HashSet;

use uuid::uuid;

use crate::entity::tag_types::{TagType, TagTypeId};

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
