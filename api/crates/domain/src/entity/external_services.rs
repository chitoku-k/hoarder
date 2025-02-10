use std::str::FromStr;

use derive_more::{Deref, Display, From};
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deref, Deserialize, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ExternalServiceId(Uuid);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalService {
    pub id: ExternalServiceId,
    pub slug: String,
    pub kind: ExternalServiceKind,
    pub name: String,
    pub base_url: Option<String>,
    pub url_pattern: Option<String>,
}

impl ExternalService {
    pub fn metadata_by_url(&self, url: &str) -> Option<ExternalMetadata> {
        let (id, creator_id) = self.url_pattern
            .as_ref()
            .and_then(|url_pattern| Regex::new(url_pattern).ok())
            .and_then(|re| re.captures(url))
            .map(|captures| {
                let id = captures.name("id").map(|c| c.as_str());
                let creator_id = captures.name("creatorId").map(|c| c.as_str());
                (id, creator_id)
            })
            .unwrap_or_default();

        ExternalMetadata::from_metadata(&self.kind, url, id, creator_id)
    }
}

#[derive(Clone, Debug, strum::Display, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ExternalServiceKind {
    Bluesky,
    Fantia,
    Mastodon,
    Misskey,
    Nijie,
    Pixiv,
    PixivFanbox,
    Pleroma,
    Seiga,
    Skeb,
    Threads,
    Website,
    X,
    Xfolio,
    #[strum(default)]
    Custom(String),
}

impl From<String> for ExternalServiceKind {
    fn from(value: String) -> Self {
        Self::try_from(value.as_str()).unwrap()
    }
}

#[derive(Clone, Debug, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
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

impl ExternalMetadata {
    const BASE_URL_BLUESKY: &str = "https://bsky.app";
    const BASE_URL_FANTIA: &str = "https://fantia.jp";
    const BASE_URL_NIJIE: &str = "https://nijie.info";
    const BASE_URL_PIXIV: &str = "https://www.pixiv.net";
    const BASE_URL_SEIGA: &str = "https://seiga.nicovideo.jp";
    const BASE_URL_SKEB: &str = "https://skeb.jp";
    const BASE_URL_THREADS: &str = "https://www.threads.net";
    const BASE_URL_X: &str = "https://x.com";
    const BASE_URL_XFOLIO: &str = "https://xfolio.jp";

    pub fn from_metadata(kind: &ExternalServiceKind, url: &str, id: Option<&str>, creator_id: Option<&str>) -> Option<Self> {
        use ExternalServiceKind::*;
        let metadata = match kind {
            Bluesky => Self::Bluesky { id: id?.to_string(), creator_id: creator_id?.to_string() },
            Fantia => Self::Fantia { id: id?.parse().ok()? },
            Mastodon => Self::Mastodon { id: id?.parse().ok()?, creator_id: creator_id?.to_string() },
            Misskey => Self::Misskey { id: id?.to_string() },
            Nijie => Self::Nijie { id: id?.parse().ok()? },
            Pixiv => Self::Pixiv { id: id?.parse().ok()? },
            PixivFanbox => Self::PixivFanbox { id: id?.parse().ok()?, creator_id: creator_id?.to_string() },
            Pleroma => Self::Pleroma { id: id?.to_string() },
            Seiga => Self::Seiga { id: id?.parse().ok()? },
            Skeb => Self::Skeb { id: id?.parse().ok()?, creator_id: creator_id?.to_string() },
            Threads => Self::Threads { id: id?.to_string(), creator_id: creator_id.map(Into::into) },
            Website => Self::Website { url: url.to_string() },
            X => Self::X { id: id?.parse().ok()?, creator_id: creator_id.map(Into::into) },
            Xfolio => Self::Xfolio { id: id?.parse().ok()?, creator_id: creator_id?.to_string() },
            Custom(..) => return None,
        };

        Some(metadata)
    }

    pub fn kind(&self) -> Option<ExternalServiceKind> {
        if let ExternalMetadata::Custom(_) = self {
            None
        } else {
            ExternalServiceKind::from_str(self.into()).ok()
        }
    }

    pub fn url(&self, base_url: Option<&str>) -> Option<String> {
        let base_url = base_url.map(|b| b.trim_end_matches("/"));

        use ExternalMetadata::*;
        let url = match self {
            Bluesky { id, creator_id } => format!("{}/profile/{creator_id}/post/{id}", base_url.unwrap_or(Self::BASE_URL_BLUESKY)),
            Fantia { id } => format!("{}/posts/{id}", base_url.unwrap_or(Self::BASE_URL_FANTIA)),
            Mastodon { id, creator_id } => format!("{}/@{creator_id}/{id}", base_url?),
            Misskey { id } => format!("{}/notes/{id}", base_url?),
            Nijie { id } => format!("{}/view.php?id={id}", base_url.unwrap_or(Self::BASE_URL_NIJIE)),
            Pixiv { id } => format!("{}/artworks/{id}", base_url.unwrap_or(Self::BASE_URL_PIXIV)),
            PixivFanbox { id, creator_id } => format!("https://{creator_id}.fanbox.cc/posts/{id}"),
            Pleroma { id } => format!("{}/notice/{id}", base_url?),
            Seiga { id } => format!("{}/seiga/im{id}", base_url.unwrap_or(Self::BASE_URL_SEIGA)),
            Skeb { id, creator_id } => format!("{}/@{creator_id}/works/{id}", base_url.unwrap_or(Self::BASE_URL_SKEB)),
            Threads { id, creator_id } => format!("{}/@{}/post/{id}", base_url.unwrap_or(Self::BASE_URL_THREADS), creator_id.as_deref().unwrap_or_default()),
            Website { url } => url.clone(),
            X { id, creator_id } => format!("{}/{}/status/{id}", base_url.unwrap_or(Self::BASE_URL_X), creator_id.as_deref().unwrap_or("i")),
            Xfolio { id, creator_id } => format!("{}/portfolio/{creator_id}/works/{id}", base_url.unwrap_or(Self::BASE_URL_XFOLIO)),
            Custom(..) => return None,
        };

        Some(url)
    }
}
