use crate::repository::objects::{ObjectOverwriteBehavior, ObjectStatus};

#[test]
fn object_overwrite_behavior_is_allowed() {
    let behavior = ObjectOverwriteBehavior::Overwrite;
    let actual = behavior.is_allowed();

    assert!(actual);

    let behavior = ObjectOverwriteBehavior::Fail;
    let actual = behavior.is_allowed();

    assert!(!actual);
}

#[test]
fn object_overwrite_behavior_is_denined() {
    let behavior = ObjectOverwriteBehavior::Overwrite;
    let actual = behavior.is_denied();

    assert!(!actual);

    let behavior = ObjectOverwriteBehavior::Fail;
    let actual = behavior.is_denied();

    assert!(actual);
}

#[test]
fn object_status_is_created() {
    let status = ObjectStatus::Created;
    let actual = status.is_created();

    assert!(actual);

    let status = ObjectStatus::Existing;
    let actual = status.is_created();

    assert!(!actual);
}

#[test]
fn object_status_is_existing() {
    let status = ObjectStatus::Created;
    let actual = status.is_existing();

    assert!(!actual);

    let status = ObjectStatus::Existing;
    let actual = status.is_existing();

    assert!(actual);
}
