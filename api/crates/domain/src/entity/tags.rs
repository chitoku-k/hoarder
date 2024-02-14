use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use derive_more::{Constructor, Deref, Display, From};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Constructor, Debug, Default, Deref, Eq, PartialEq, Serialize)]
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn tag_id_root() {
        let actual = TagId::root();

        assert!(actual.0.is_nil());
    }

    #[test]
    fn tag_id_is_root() {
        let actual = TagId::root();

        assert!(actual.is_root());
    }

    #[test]
    fn tag_depth_parent() {
        let depth = TagDepth::new(1, 2);
        let actual = depth.parent();

        assert_eq!(actual, 1);
    }

    #[test]
    fn tag_depth_children() {
        let depth = TagDepth::new(1, 2);
        let actual = depth.children();

        assert_eq!(actual, 2);
    }

    #[test]
    fn tag_depth_has_parent() {
        let depth = TagDepth::new(1, 2);
        let actual = depth.has_parent();

        assert!(actual);
    }

    #[test]
    fn tag_depth_has_no_parent() {
        let depth = TagDepth::new(0, 2);
        let actual = depth.has_parent();

        assert!(!actual);
    }

    #[test]
    fn tag_depth_has_children() {
        let depth = TagDepth::new(1, 2);
        let actual = depth.has_children();

        assert!(actual);
    }

    #[test]
    fn tag_depth_has_no_children() {
        let depth = TagDepth::new(1, 0);
        let actual = depth.has_children();

        assert!(!actual);
    }

    #[test]
    fn alias_set_add_all() {
        let mut actual = AliasSet::new(BTreeSet::from(["baz".to_string()]));
        actual.add_all(["foo".to_string(), "bar".to_string()]);

        assert_eq!(actual, AliasSet::new(BTreeSet::from(["foo".to_string(), "bar".to_string(), "baz".to_string()])));
    }

    #[test]
    fn alias_set_remove_all() {
        let mut actual = AliasSet::new(BTreeSet::from(["foo".to_string(), "bar".to_string(), "baz".to_string()]));
        actual.remove_all(["foo".to_string(), "bar".to_string()]);

        assert_eq!(actual, AliasSet::new(BTreeSet::from(["baz".to_string()])));
    }

    #[test]
    fn alias_set_into_inner() {
        let set = AliasSet::new(BTreeSet::from(["foo".to_string(), "bar".to_string(), "baz".to_string()]));
        let actual = set.into_inner();

        assert_eq!(actual, BTreeSet::from(["foo".to_string(), "bar".to_string(), "baz".to_string()]));
    }

    #[test]
    fn alias_set_from_strings() {
        let actual = AliasSet::from(vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]);

        assert_eq!(actual, AliasSet::new(BTreeSet::from(["foo".to_string(), "bar".to_string(), "baz".to_string()])));
    }
}
