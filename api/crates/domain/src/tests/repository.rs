use crate::repository::DeleteResult;

#[test]
fn delete_result_is_not_found() {
    let result = DeleteResult::NotFound;
    let actual = result.is_not_found();

    assert!(actual);

    let result = DeleteResult::Deleted(1);
    let actual = result.is_not_found();

    assert!(!actual);
}

#[test]
fn delete_result_is_deleted() {
    let result = DeleteResult::NotFound;
    let actual = result.is_deleted();

    assert!(!actual);

    let result = DeleteResult::Deleted(1);
    let actual = result.is_deleted();

    assert!(actual);
}
