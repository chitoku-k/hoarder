use std::collections::BTreeSet;

use async_graphql::{
    connection::{CursorType, DefaultEdgeName, Edge, EmptyFields},
    InputObject, Lookahead, SimpleObject,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use domain::entity::{
    tag_types::{self, TagTypeId},
    tags::{self, TagDepth, TagId},
};
use uuid::Uuid;

use crate::error::ErrorKind;

/// A tag represents a user-friendly and hierarchical attribute attached to media.
#[derive(SimpleObject)]
pub struct Tag {
    /// The ID of the Tag object.
    id: Uuid,
    /// The name of the tag.
    name: String,
    /// The kana of the tag.
    kana: String,
    /// The list of aliases for the tag.
    aliases: BTreeSet<String>,
    /// The parent node of the tag.
    parent: Option<Box<Tag>>,
    /// The child nodes of the tag.
    children: Vec<Tag>,
    /// The date at which the tag was created.
    created_at: DateTime<Utc>,
    /// The date at which the tag was updated.
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub(crate) struct TagCursor(String, Uuid);

/// A tag type represents a type of the tag being attached to media describing how
/// the tag corresponds to the media.
#[derive(SimpleObject)]
pub struct TagType {
    /// The ID of the TagType object.
    id: Uuid,
    /// The short and user-friendly name that uniquely identifies the tag type.
    slug: String,
    /// The name of the tag type.
    name: String,
    /// The kana of the tag type.
    kana: String,
}

/// A tag tag type is a pair of a tag and a tag type.
#[derive(Constructor, SimpleObject)]
pub struct TagTagType {
    /// The tag.
    tag: Tag,
    /// The tag type.
    #[graphql(name = "type")]
    tag_type: TagType,
}

/// A tag tag type input is a pair of the ID of a tag and tag type.
#[derive(Clone, Copy, InputObject)]
pub struct TagTagTypeInput {
    /// The ID of the tag.
    tag_id: Uuid,
    /// The ID of the tag type.
    tag_type_id: Uuid,
}

impl From<tags::Tag> for Tag {
    fn from(tag: tags::Tag) -> Self {
        let parent = tag.parent
            .map(|p| (*p).into())
            .map(Box::new);

        let children = tag.children
            .into_iter()
            .map(Into::into)
            .collect();

        Self {
            id: *tag.id,
            name: tag.name,
            kana: tag.kana,
            aliases: tag.aliases.into_inner(),
            parent,
            children,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }
    }
}

impl TagCursor {
    const DELIMITER: char = '\x00';

    pub fn into_inner(self) -> (String, TagId) {
        (self.0, self.1.into())
    }
}

impl CursorType for TagCursor {
    type Error = ErrorKind;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let bin = BASE64_STANDARD.decode(s).map_err(|_| ErrorKind::CursorInvalid)?;
        let str = String::from_utf8(bin).map_err(|_| ErrorKind::CursorInvalid)?;

        let (kana, uuid) = str.split_once(Self::DELIMITER).ok_or(ErrorKind::CursorInvalid)?;
        let kana = kana.to_string();
        let uuid = Uuid::parse_str(uuid).map_err(|_| ErrorKind::CursorInvalid)?;

        Ok(TagCursor::new(kana, uuid))
    }

    fn encode_cursor(&self) -> String {
        let str = format!("{}{}{}", &self.0, Self::DELIMITER, &self.1);

        BASE64_STANDARD.encode(str)
    }
}

impl From<Tag> for Edge<TagCursor, Tag, EmptyFields, DefaultEdgeName> {
    fn from(tag: Tag) -> Self {
        let cursor = TagCursor::new(tag.kana.clone(), tag.id);
        Edge::new(cursor, tag)
    }
}

impl From<tag_types::TagType> for TagType {
    fn from(tag_type: tag_types::TagType) -> Self {
        Self {
            id: *tag_type.id,
            slug: tag_type.slug,
            name: tag_type.name,
            kana: tag_type.kana,
        }
    }
}

impl From<TagTagTypeInput> for (TagId, TagTypeId) {
    fn from(input: TagTagTypeInput) -> Self {
        (input.tag_id.into(), input.tag_type_id.into())
    }
}

pub fn get_tag_depth(root: &Lookahead<'_>) -> TagDepth {
    let mut parent = 0;
    let mut children = 0;

    let mut parent_look_ahead = root.field("parent");
    let mut children_look_ahead = root.field("children");

    loop {
        match (parent_look_ahead.exists(), children_look_ahead.exists()) {
            (false, false) => break,
            (true, true) => {
                parent += 1;
                children += 1;
            },
            (true, false) => {
                parent += 1;
            },
            (false, true) => {
                children += 1;
            },
        }

        parent_look_ahead = parent_look_ahead.field("parent");
        children_look_ahead = children_look_ahead.field("children");
    }

    TagDepth::new(parent, children)
}
