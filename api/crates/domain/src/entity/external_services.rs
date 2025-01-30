use derive_more::{Deref, Display, From};
use regex::Regex;
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

impl ExternalMetadata {
    const KIND_BLUESKY: &str = "bluesky";
    const KIND_FANTIA: &str = "fantia";
    const KIND_MASTODON: &str = "mastodon";
    const KIND_MISSKEY: &str = "misskey";
    const KIND_NIJIE: &str = "nijie";
    const KIND_PIXIV: &str = "pixiv";
    const KIND_PIXIV_FANBOX: &str = "pixiv_fanbox";
    const KIND_PLEROMA: &str = "pleroma";
    const KIND_SEIGA: &str = "seiga";
    const KIND_SKEB: &str = "skeb";
    const KIND_THREADS: &str = "threads";
    const KIND_WEBSITE: &str = "website";
    const KIND_X: &str = "x";
    const KIND_XFOLIO: &str = "xfolio";

    const BASE_URL_BLUESKY: &str = "https://bsky.app";
    const BASE_URL_FANTIA: &str = "https://fantia.jp";
    const BASE_URL_NIJIE: &str = "https://nijie.info";
    const BASE_URL_PIXIV: &str = "https://www.pixiv.net";
    const BASE_URL_SEIGA: &str = "https://seiga.nicovideo.jp";
    const BASE_URL_SKEB: &str = "https://skeb.jp";
    const BASE_URL_THREADS: &str = "https://www.threads.net";
    const BASE_URL_X: &str = "https://x.com";
    const BASE_URL_XFOLIO: &str = "https://xfolio.jp";

    pub fn from_metadata(kind: &str, url: &str, id: Option<&str>, creator_id: Option<&str>) -> Option<Self> {
        match kind {
            Self::KIND_BLUESKY => Some(Self::Bluesky { id: id?.to_string(), creator_id: creator_id?.to_string() }),
            Self::KIND_FANTIA => Some(Self::Fantia { id: id?.parse().ok()? }),
            Self::KIND_MASTODON => Some(Self::Mastodon { id: id?.parse().ok()?, creator_id: creator_id?.to_string() }),
            Self::KIND_MISSKEY => Some(Self::Misskey { id: id?.to_string() }),
            Self::KIND_NIJIE => Some(Self::Nijie { id: id?.parse().ok()? }),
            Self::KIND_PIXIV => Some(Self::Pixiv { id: id?.parse().ok()? }),
            Self::KIND_PIXIV_FANBOX => Some(Self::PixivFanbox { id: id?.parse().ok()?, creator_id: creator_id?.to_string() }),
            Self::KIND_PLEROMA => Some(Self::Pleroma { id: id?.to_string() }),
            Self::KIND_SEIGA => Some(Self::Seiga { id: id?.parse().ok()? }),
            Self::KIND_SKEB => Some(Self::Skeb { id: id?.parse().ok()?, creator_id: creator_id?.to_string() }),
            Self::KIND_THREADS => Some(Self::Threads { id: id?.to_string(), creator_id: creator_id.map(Into::into) }),
            Self::KIND_WEBSITE => Some(Self::Website { url: url.to_string() }),
            Self::KIND_X => Some(Self::X { id: id?.parse().ok()?, creator_id: creator_id.map(Into::into) }),
            Self::KIND_XFOLIO => Some(Self::Xfolio { id: id?.parse().ok()?, creator_id: creator_id?.to_string() }),
            _ => None,
        }
    }

    pub fn kind(&self) -> Option<&'static str> {
        match self {
            Self::Bluesky { .. } => Some(Self::KIND_BLUESKY),
            Self::Fantia { .. } => Some(Self::KIND_FANTIA),
            Self::Mastodon { .. } => Some(Self::KIND_MASTODON),
            Self::Misskey { .. } => Some(Self::KIND_MISSKEY),
            Self::Nijie { .. } => Some(Self::KIND_NIJIE),
            Self::Pixiv { .. } => Some(Self::KIND_PIXIV),
            Self::PixivFanbox { .. } => Some(Self::KIND_PIXIV_FANBOX),
            Self::Pleroma { .. } => Some(Self::KIND_PLEROMA),
            Self::Seiga { .. } => Some(Self::KIND_SEIGA),
            Self::Skeb { .. } => Some(Self::KIND_SKEB),
            Self::Threads { .. } => Some(Self::KIND_THREADS),
            Self::Website { .. } => Some(Self::KIND_WEBSITE),
            Self::X { .. } => Some(Self::KIND_X),
            Self::Xfolio { .. } => Some(Self::KIND_XFOLIO),
            Self::Custom(..) => None,
        }
    }

    pub fn url(&self, base_url: Option<&str>) -> Option<String> {
        let base_url = base_url.map(|b| b.trim_end_matches("/"));
        match self {
            Self::Bluesky { id, creator_id } => Some(format!("{}/profile/{creator_id}/post/{id}", base_url.unwrap_or(Self::BASE_URL_BLUESKY))),
            Self::Fantia { id } => Some(format!("{}/posts/{id}", base_url.unwrap_or(Self::BASE_URL_FANTIA))),
            Self::Mastodon { id, creator_id } => Some(format!("{}/@{creator_id}/{id}", base_url?)),
            Self::Misskey { id } => Some(format!("{}/notes/{id}", base_url?)),
            Self::Nijie { id } => Some(format!("{}/view.php?id={id}", base_url.unwrap_or(Self::BASE_URL_NIJIE))),
            Self::Pixiv { id } => Some(format!("{}/artworks/{id}", base_url.unwrap_or(Self::BASE_URL_PIXIV))),
            Self::PixivFanbox { id, creator_id } => Some(format!("https://{creator_id}.fanbox.cc/posts/{id}")),
            Self::Pleroma { id } => Some(format!("{}/notice/{id}", base_url?)),
            Self::Seiga { id } => Some(format!("{}/seiga/im{id}", base_url.unwrap_or(Self::BASE_URL_SEIGA))),
            Self::Skeb { id, creator_id } => Some(format!("{}/@{creator_id}/works/{id}", base_url.unwrap_or(Self::BASE_URL_SKEB))),
            Self::Threads { id, creator_id } => Some(format!("{}/@{}/post/{id}", base_url.unwrap_or(Self::BASE_URL_THREADS), creator_id.as_deref().unwrap_or_default())),
            Self::Website { url } => Some(url.clone()),
            Self::X { id, creator_id } => Some(format!("{}/{}/status/{id}", base_url.unwrap_or(Self::BASE_URL_X), creator_id.as_deref().unwrap_or("i"))),
            Self::Xfolio { id, creator_id } => Some(format!("{}/portfolio/{creator_id}/works/{id}", base_url.unwrap_or(Self::BASE_URL_XFOLIO))),
            Self::Custom(..) => None,
        }
    }
}
