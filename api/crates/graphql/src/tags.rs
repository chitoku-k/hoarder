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

#[derive(SimpleObject)]
pub struct Tag {
    id: Uuid,
    name: String,
    kana: String,
    aliases: BTreeSet<String>,
    parent: Option<Box<Tag>>,
    children: Vec<Tag>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub(crate) struct TagCursor(String, Uuid);

#[derive(SimpleObject)]
pub struct TagType {
    id: Uuid,
    slug: String,
    name: String,
    kana: String,
}

#[derive(Constructor, SimpleObject)]
pub struct TagTagType {
    tag: Tag,
    #[graphql(name = "type")]
    tag_type: TagType,
}

#[derive(Clone, Copy, InputObject)]
pub struct TagTagTypeInput {
    tag_id: Uuid,
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use uuid::uuid;

    use super::*;

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
}
