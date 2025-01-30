use async_graphql::connection::CursorType;
use chrono::{TimeZone, Utc};
use domain::entity::media::MediumId;
use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::media::MediumCursor;

#[test]
fn medium_cursor_into_inner() {
    let cursor = MediumCursor::new(
        Utc.with_ymd_and_hms(2022, 1, 1, 3, 4, 15).unwrap(),
        uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"),
    );
    let actual = cursor.into_inner();

    assert_eq!(
        actual,
        (
            Utc.with_ymd_and_hms(2022, 1, 1, 3, 4, 15).unwrap(),
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        ),
    );
}

#[test]
fn media_cursor_encode_cursor_succeeds() {
    let cursor = MediumCursor::new(
        Utc.with_ymd_and_hms(2022, 1, 1, 3, 4, 15).unwrap(),
        uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"),
    );
    let actual = cursor.encode_cursor();

    assert_eq!(actual, "MjAyMi0wMS0wMVQwMzowNDoxNSswMDowMAA2MzU2NTAzZC02YWI2LTRlMzktYmI4Ni0zMzExMjE5YzdmZDE=".to_string());
}

#[test]
fn media_cursor_decode_cursor_succeeds() {
    let actual = MediumCursor::decode_cursor("MjAyMi0wMS0wMVQwMzowNDoxNSswMDowMAA2MzU2NTAzZC02YWI2LTRlMzktYmI4Ni0zMzExMjE5YzdmZDE=").unwrap();

    assert_eq!(
        actual,
        MediumCursor::new(
            Utc.with_ymd_and_hms(2022, 1, 1, 3, 4, 15).unwrap(),
            uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"),
        ),
    );
}

#[test]
fn media_cursor_decode_cursor_fails() {
    let actual = MediumCursor::decode_cursor("====");

    assert!(actual.is_err());
}
