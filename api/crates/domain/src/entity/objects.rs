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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
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

#[cfg(test)]
mod tests {
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
}
