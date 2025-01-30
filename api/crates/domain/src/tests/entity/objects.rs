use pretty_assertions::{assert_eq, assert_matches};

use crate::{entity::objects::{EntryUrl, EntryUrlPath}, error::ErrorKind};

#[test]
fn entry_url_from_path_str() {
    let actual = EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual, EntryUrl::from("file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/77777777-7777-7777-7777-777777777777.png".to_string()));
}

#[test]
fn entry_url_to_path_string_prefix_mismatch() {
    let url = EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string());

    let actual = url.to_path_string("s3://").unwrap_err();
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlUnsupported { url } if url == "file:///77777777-7777-7777-7777-777777777777.png");
}

#[test]
fn entry_url_to_path_string_utf8_valid() {
    let url = EntryUrl::from("file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/77777777-7777-7777-7777-777777777777.png".to_string());

    let actual = url.to_path_string("file://").unwrap();
    assert_eq!(actual, "/ゆるゆり/77777777-7777-7777-7777-777777777777.png".to_string());
}

#[test]
fn entry_url_to_path_string_utf8_invalid() {
    let url = EntryUrl::from("file:///%80.png".to_string());

    let actual = url.to_path_string("file://").unwrap_err();
    assert_matches!(actual.kind(), ErrorKind::ObjectPathInvalid);
}

#[test]
fn entry_url_into_inner() {
    let url = EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());

    let actual = url.into_inner();
    assert_eq!(actual, "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());
}

#[test]
fn entry_url_path_to_url() {
    let path = EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());

    let actual = path.to_url("file");
    assert_eq!(actual, EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()));
}

#[test]
fn entry_url_path_into_inner() {
    let url = EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());

    let actual = url.into_inner();
    assert_eq!(actual, "/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());
}
