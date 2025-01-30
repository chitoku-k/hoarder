use async_graphql::connection::CursorType;
use domain::entity::tags::TagId;
use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::tags::TagCursor;

#[test]
fn tag_cursor_into_inner() {
    let cursor = TagCursor::new(
        "ななもりちゅうごらくぶ".to_string(),
        uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"),
    );
    let actual = cursor.into_inner();

    assert_eq!(
        actual,
        (
            "ななもりちゅうごらくぶ".to_string(),
            TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
        ),
    );
}

#[test]
fn tag_cursor_into_encode_cursor_succeeds() {
    let cursor = TagCursor::new(
        "ななもりちゅうごらくぶ".to_string(),
        uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"),
    );
    let actual = cursor.encode_cursor();

    assert_eq!(actual, "44Gq44Gq44KC44KK44Gh44KF44GG44GU44KJ44GP44G2ADEyYzQxMDFlLTcyMmYtNDE3Mi05ZmUyLTc4NjJlYmJjOGZjNQ==".to_string());
}

#[test]
fn tag_cursor_into_decode_cursor_succeeds() {
    let actual = TagCursor::decode_cursor("44Gq44Gq44KC44KK44Gh44KF44GG44GU44KJ44GP44G2ADEyYzQxMDFlLTcyMmYtNDE3Mi05ZmUyLTc4NjJlYmJjOGZjNQ==").unwrap();

    assert_eq!(
        actual,
        TagCursor::new(
            "ななもりちゅうごらくぶ".to_string(),
            uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"),
        ),
    );
}

#[test]
fn tag_cursor_into_decode_cursor_fails() {
    let actual = TagCursor::decode_cursor("====");

    assert!(actual.is_err());
}
