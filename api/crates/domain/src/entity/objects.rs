use chrono::{DateTime, Utc};
use derive_more::{Constructor, Deref, Display, From};
use serde::{Deserialize, Serialize};

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
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl EntryUrlPath {
    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn to_url(&self, scheme: &str) -> EntryUrl {
        let path = &self.0;
        let url = format!("{scheme}://{path}");
        EntryUrl::from(url)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::entity::objects::{EntryUrl, EntryUrlPath};

    #[test]
    fn entry_url_into_inner() {
        let url = EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());

        let actual = url.into_inner();
        assert_eq!(actual, "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());
    }

    #[test]
    fn entry_url_path_into_inner() {
        let url = EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());

        let actual = url.into_inner();
        assert_eq!(actual, "/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());
    }

    #[test]
    fn entry_url_path_to_url() {
        let path = EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string());

        let actual = path.to_url("file");
        assert_eq!(actual, EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()));
    }
}
