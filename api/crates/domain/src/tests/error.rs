use pretty_assertions::{assert_eq, assert_matches};

use crate::error::{Error, ErrorKind};

#[test]
fn kind_succeeds() {
    let error = Error::new(ErrorKind::Other, "error communicating with database");
    let actual = error.kind();
    assert_matches!(actual, ErrorKind::Other);

    let error = Error::other("error communicating with database");
    let actual = error.kind();
    assert_matches!(actual, ErrorKind::Other);
}

#[test]
fn error_succeeds() {
    let error = Error::new(ErrorKind::Other, "error communicating with database");
    let actual = error.error().unwrap();
    assert_eq!(actual.to_string(), "error communicating with database");

    let error = Error::other("error communicating with database");
    let actual = error.error().unwrap();
    assert_eq!(actual.to_string(), "error communicating with database");
}

#[test]
fn into_inner_succeeds() {
    let error = Error::new(ErrorKind::Other, "error communicating with database");
    let actual = error.into_inner();
    assert_matches!(actual, (ErrorKind::Other, Some(e)) if e.to_string() == "error communicating with database");

    let error = Error::other("error communicating with database");
    let actual = error.into_inner();
    assert_matches!(actual, (ErrorKind::Other, Some(e)) if e.to_string() == "error communicating with database");
}
