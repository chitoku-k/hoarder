use std::collections::BTreeSet;

use pretty_assertions::assert_eq;

use crate::entity::tags::{AliasSet, TagDepth, TagId};

#[test]
fn tag_id_root() {
    let actual = TagId::root();

    assert!(actual.is_nil());
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
