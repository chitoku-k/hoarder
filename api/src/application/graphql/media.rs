use anyhow::Context;
use async_graphql::{
    connection::{CursorType, DefaultEdgeName, Edge, EmptyFields},
    SimpleObject,
};
use chrono::NaiveDateTime;
use derive_more::Constructor;
use uuid::Uuid;

use crate::{
    application::graphql::{
        query::QueryError,
        replicas::Replica,
        sources::Source,
        tags::TagTagType,
    },
    domain::entity::media::{self, MediumId},
};

#[derive(SimpleObject)]
pub struct Medium {
    id: Uuid,
    sources: Vec<Source>,
    tags: Vec<TagTagType>,
    replicas: Vec<Replica>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub struct MediumCursor(NaiveDateTime, Uuid);

impl TryFrom<media::Medium> for Medium {
    type Error = anyhow::Error;

    fn try_from(medium: media::Medium) -> anyhow::Result<Self> {
        let sources: anyhow::Result<_> = medium.sources
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
    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.f";

    pub fn into_inner(self) -> (NaiveDateTime, MediumId) {
        (self.0, self.1.into())
    }
}

impl CursorType for MediumCursor {
    type Error = anyhow::Error;

    fn decode_cursor(s: &str) -> anyhow::Result<Self> {
        let bin = base64::decode(s)?;
        let str = String::from_utf8(bin)?;

        let (datetime, uuid) = str.split_once(Self::DELIMITER).context(QueryError::InvalidCursor)?;
        let datetime = NaiveDateTime::parse_from_str(datetime, Self::FORMAT)?;
        let uuid = Uuid::parse_str(uuid)?;

        Ok(MediumCursor::new(datetime, uuid))
    }

    fn encode_cursor(&self) -> String {
        let datetime = self.0.format(Self::FORMAT);
        let uuid = self.1;
        let str = format!("{}{}{}", datetime, Self::DELIMITER, uuid);

        base64::encode(&str)
    }
}

impl TryFrom<media::Medium> for Edge<MediumCursor, Medium, EmptyFields, DefaultEdgeName> {
    type Error = anyhow::Error;

    fn try_from(medium: media::Medium) -> anyhow::Result<Self> {
        let cursor = MediumCursor::new(medium.created_at, *medium.id);
        let medium = Medium::try_from(medium)?;
        Ok(Edge::new(cursor, medium))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use uuid::uuid;

    use super::*;

    #[test]
    fn medium_cursor_into_inner() {
        let cursor = MediumCursor::new(
            NaiveDate::from_ymd(2022, 1, 1).and_hms(3, 4, 15),
            uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"),
        );
        let actual = cursor.into_inner();

        assert_eq!(
            actual,
            (
                NaiveDate::from_ymd(2022, 1, 1).and_hms(3, 4, 15),
                MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            ),
        );
    }

    #[test]
    fn media_cursor_encode_cursor_succeeds() {
        let cursor = MediumCursor::new(
            NaiveDate::from_ymd(2022, 1, 1).and_hms(3, 4, 15),
            uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"),
        );
        let actual = cursor.encode_cursor();

        assert_eq!(actual, "MjAyMi0wMS0wMVQwMzowNDoxNQA2MzU2NTAzZC02YWI2LTRlMzktYmI4Ni0zMzExMjE5YzdmZDE=".to_string());
    }

    #[test]
    fn media_cursor_decode_cursor_succeeds() {
        let actual = MediumCursor::decode_cursor("MjAyMi0wMS0wMVQwMzowNDoxNQA2MzU2NTAzZC02YWI2LTRlMzktYmI4Ni0zMzExMjE5YzdmZDE=").unwrap();

        assert_eq!(
            actual,
            MediumCursor::new(
                NaiveDate::from_ymd(2022, 1, 1).and_hms(3, 4, 15),
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
