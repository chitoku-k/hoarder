use std::collections::BTreeSet;

use anyhow::Context;
use async_graphql::{
    connection::{CursorType, DefaultEdgeName, Edge, EmptyFields},
    InputObject, Lookahead, SimpleObject,
};
use chrono::NaiveDateTime;
use derive_more::Constructor;
use uuid::Uuid;

use crate::{
    application::graphql::query::QueryError,
    domain::entity::{
        tag_types::{self, TagTypeId},
        tags::{self, TagDepth, TagId},
    },
};

#[derive(SimpleObject)]
pub struct Tag {
    id: Uuid,
    name: String,
    kana: String,
    aliases: BTreeSet<String>,
    parent: Option<Box<TagParent>>,
    children: Vec<TagChild>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(SimpleObject)]
pub struct TagParent {
    id: Uuid,
    name: String,
    kana: String,
    aliases: BTreeSet<String>,
    parent: Option<Box<Self>>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(SimpleObject)]
pub struct TagChild {
    id: Uuid,
    name: String,
    kana: String,
    aliases: BTreeSet<String>,
    children: Vec<Self>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Clone, Constructor)]
pub struct TagCursor(String, Uuid);

#[derive(SimpleObject)]
pub struct TagType {
    id: Uuid,
    slug: String,
    name: String,
}

#[derive(Constructor, SimpleObject)]
pub struct TagTagType {
    tag: Tag,
    #[graphql(name = "type")]
    tag_type: TagType,
}

#[derive(InputObject)]
pub struct TagTagTypeInput {
    pub tag_id: Uuid,
    pub tag_type_id: Uuid,
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

impl From<tags::Tag> for TagParent {
    fn from(tag: tags::Tag) -> Self {
        let parent = tag.parent
            .map(|p| (*p).into())
            .map(Box::new);

        Self {
            id: *tag.id,
            name: tag.name,
            kana: tag.kana,
            aliases: tag.aliases.into_inner(),
            parent,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }
    }
}

impl From<tags::Tag> for TagChild {
    fn from(tag: tags::Tag) -> Self {
        let children = tag.children
            .into_iter()
            .map(Into::into)
            .collect();

        Self {
            id: *tag.id,
            name: tag.name,
            kana: tag.kana,
            aliases: tag.aliases.into_inner(),
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
    type Error = anyhow::Error;

    fn decode_cursor(s: &str) -> anyhow::Result<Self> {
        let bin = base64::decode(s)?;
        let str = String::from_utf8(bin)?;

        let (kana, uuid) = str.split_once(Self::DELIMITER).context(QueryError::InvalidCursor)?;
        let kana = kana.to_string();
        let uuid = Uuid::parse_str(uuid)?;

        Ok(TagCursor::new(kana, uuid))
    }

    fn encode_cursor(&self) -> String {
        let str = format!("{}{}{}", &self.0, Self::DELIMITER, &self.1);

        base64::encode(&str)
    }
}

impl From<tags::Tag> for Edge<TagCursor, Tag, EmptyFields, DefaultEdgeName> {
    fn from(tag: tags::Tag) -> Self {
        let cursor = TagCursor::new(tag.kana.clone(), *tag.id);
        let tag = Tag::from(tag);
        Edge::new(cursor, tag)
    }
}

impl From<tag_types::TagType> for TagType {
    fn from(tag_type: tag_types::TagType) -> Self {
        Self {
            id: *tag_type.id,
            slug: tag_type.slug,
            name: tag_type.name,
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
