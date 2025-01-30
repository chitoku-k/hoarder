use chrono::{DateTime, Utc};
use derive_more::{Constructor, Deref, Display, From};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind, Result};

#[derive(Clone, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, PartialEq, Serialize)]
pub struct EntryUrl(String);

#[derive(Clone, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, PartialEq, Serialize)]
pub struct EntryUrlPath(String);

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub struct Entry {
    pub name: String,
    pub url: Option<EntryUrl>,
    pub kind: EntryKind,
    pub metadata: Option<EntryMetadata>,
}

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub struct EntryMetadata {
    pub size: u64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub accessed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    Container,
    Object,
    Unknown,
}

impl EntryUrl {
    const RFC3986_PATH: &'static AsciiSet = &CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'#')
        .add(b'%')
        .add(b'<')
        .add(b'>')
        .add(b'?')
        .add(b'^')
        .add(b'`')
        .add(b'{')
        .add(b'}');

    pub fn from_path_str(prefix: &str, path: &str) -> Self {
        let url = format!(
            "{}{}",
            prefix,
            utf8_percent_encode(path, Self::RFC3986_PATH),
        );

        Self::from(url)
    }

    pub fn to_path_string(&self, prefix: &str) -> Result<String> {
        let path = self
            .strip_prefix(prefix)
            .ok_or_else(|| ErrorKind::ObjectUrlUnsupported { url: self.to_string() })?;

        let path = percent_decode_str(path)
            .decode_utf8()
            .map_err(|e| Error::new(ErrorKind::ObjectPathInvalid, e))?
            .to_string();

        Ok(path)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl EntryUrlPath {
    pub fn to_url(&self, scheme: &str) -> EntryUrl {
        let path = &self.0;
        let url = format!("{scheme}://{path}");
        EntryUrl::from(url)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}
