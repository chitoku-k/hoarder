use std::fmt::Write;

use anyhow::anyhow;
use indoc::indoc;
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

#[test]
fn debug_fmt_succeeds() {
    let error = Error::new(ErrorKind::Other, "error communicating with database");
    let actual = {
        let mut s = String::new();
        write!(&mut s, "{:?}", &error).unwrap();
        s
    };
    assert_eq!(actual, "error communicating with database");

    let error = Error::other("error communicating with database");
    let actual = {
        let mut s = String::new();
        write!(&mut s, "{:?}", &error).unwrap();
        s
    };
    assert_eq!(actual, "error communicating with database");
}

#[test]
fn debug_fmt_with_details_succeeds() {
    let error = Error::from(ErrorKind::Other);
    let actual = {
        let mut s = String::new();
        write!(&mut s, "{:?}", &error).unwrap();
        s
    };
    assert_eq!(actual, indoc! {"
        other error
        Details:
              Other"
    });
}

#[test]
fn debug_fmt_with_single_source_succeeds() {
    let error = Error::new(ErrorKind::Other, anyhow!("error communicating with database").context("error fetching data"));
    let actual = {
        let mut s = String::new();
        write!(&mut s, "{:?}", &error).unwrap();
        s
    };
    assert_eq!(actual, indoc! {"
        error fetching data
        Caused by:
              error communicating with database"
    });

    let error = Error::other(anyhow!("error communicating with database").context("error fetching data"));
    let actual = {
        let mut s = String::new();
        write!(&mut s, "{:?}", &error).unwrap();
        s
    };
    assert_eq!(actual, indoc! {"
        error fetching data
        Caused by:
              error communicating with database"
    });
}

#[test]
fn debug_fmt_with_multiple_sources_succeeds() {
    let error = Error::new(ErrorKind::Other, anyhow!("error communicating with database").context("error fetching data").context("query error"));
    let actual = {
        let mut s = String::new();
        write!(&mut s, "{:?}", &error).unwrap();
        s
    };
    assert_eq!(actual, indoc! {"
        query error
        Caused by:
           0: error fetching data
           1: error communicating with database"
    });

    let error = Error::other(anyhow!("error communicating with database").context("error fetching data").context("query error"));
    let actual = {
        let mut s = String::new();
        write!(&mut s, "{:?}", &error).unwrap();
        s
    };
    assert_eq!(actual, indoc! {"
        query error
        Caused by:
           0: error fetching data
           1: error communicating with database"
    });
}
