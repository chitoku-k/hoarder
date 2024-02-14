use async_graphql::{
    connection::{CursorType, DefaultEdgeName, Edge, EmptyFields},
    SimpleObject,
};
use base64::prelude::{BASE64_STANDARD, Engine};
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use domain::entity::media::{self, MediumId};
use uuid::Uuid;

use crate::{
    error::ErrorKind,
    replicas::Replica,
    sources::Source,
    tags::TagTagType,
};

#[derive(SimpleObject)]
pub(crate) struct Medium {
    id: Uuid,
    sources: Vec<Source>,
    tags: Vec<TagTagType>,
    replicas: Vec<Replica>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub(crate) struct MediumCursor(DateTime<Utc>, Uuid);

impl TryFrom<media::Medium> for Medium {
    type Error = ErrorKind;

    fn try_from(medium: media::Medium) -> Result<Self, Self::Error> {
        let sources: Result<_, _> = medium.sources
            .into_iter()
            .map(TryInto::try_into)
            .collect();

        let tags = medium.tags
            .into_iter()
            .flat_map(|(tag_type, tags)| {
                tags
                    .into_iter()
                    .map(move |tag| TagTagType::new(tag.into(), tag_type.clone().into()))
            })
            .collect();

        let replicas = medium.replicas
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(Self {
            id: *medium.id,
            sources: sources?,
            tags,
            replicas,
            created_at: medium.created_at,
            updated_at: medium.updated_at,
        })
    }
}

impl MediumCursor {
    const DELIMITER: char = '\x00';

    pub fn into_inner(self) -> (DateTime<Utc>, MediumId) {
        (self.0, self.1.into())
    }
}

impl CursorType for MediumCursor {
    type Error = ErrorKind;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let bin = BASE64_STANDARD.decode(s).map_err(|_| ErrorKind::CursorInvalid)?;
        let str = String::from_utf8(bin).map_err(|_| ErrorKind::CursorInvalid)?;

        let (datetime, uuid) = str.split_once(Self::DELIMITER).ok_or(ErrorKind::CursorInvalid)?;
        let datetime = DateTime::parse_from_rfc3339(datetime).map_err(|_| ErrorKind::CursorInvalid)?.into();
        let uuid = Uuid::parse_str(uuid).map_err(|_| ErrorKind::CursorInvalid)?;

        Ok(MediumCursor::new(datetime, uuid))
    }

    fn encode_cursor(&self) -> String {
        let datetime = self.0.to_rfc3339();
        let uuid = self.1;
        let str = format!("{}{}{}", datetime, Self::DELIMITER, uuid);

        BASE64_STANDARD.encode(str)
    }
}

impl From<Medium> for Edge<MediumCursor, Medium, EmptyFields, DefaultEdgeName> {
    fn from(medium: Medium) -> Self {
        let cursor = MediumCursor::new(medium.created_at, medium.id);
        Edge::new(cursor, medium)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;
    use uuid::uuid;

    use super::*;

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
}
