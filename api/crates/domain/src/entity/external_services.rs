use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ExternalServiceId(Uuid);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalService {
    pub id: ExternalServiceId,
    pub slug: String,
    pub kind: String,
    pub name: String,
    pub base_url: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExternalMetadata {
    Bluesky { id: String, creator_id: String },
    Fantia { id: u64 },
    Mastodon { id: u64, creator_id: String },
    Misskey { id: String },
    Nijie { id: u64 },
    Pixiv { id: u64 },
    PixivFanbox { id: u64, creator_id: String },
    Pleroma { id: String },
    Seiga { id: u64 },
    Skeb { id: u64, creator_id: String },
    Threads { id: String, creator_id: Option<String> },
    Website { url: String },
    X { id: u64, creator_id: Option<String> },
    Xfolio { id: u64, creator_id: String },
    Custom(String),
}
