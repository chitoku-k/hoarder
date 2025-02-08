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
    error::{Error, ErrorKind},
    replicas::Replica,
    sources::Source,
    tags::TagTagType,
};

/// A medium represents a set of sources, tags, and replicas.
#[derive(SimpleObject)]
pub(crate) struct Medium {
    /// The ID of the Medium object.
    id: Uuid,
    /// The sources attached to the medium.
    sources: Vec<Source>,
    /// The tags attached to the medium.
    tags: Vec<TagTagType>,
    /// The replicas that belongs to the medium.
    replicas: Vec<Replica>,
    /// The date at which the medium was created.
    created_at: DateTime<Utc>,
    /// The date at which the medium was updated.
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub(crate) struct MediumCursor(DateTime<Utc>, Uuid);

impl TryFrom<media::Medium> for Medium {
    type Error = Error;

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
    type Error = Error;

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
