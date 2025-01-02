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
        match *self {
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
}

#[cfg(test)]
mod tests {
    use crate::entity::external_services::ExternalMetadata;

    use pretty_assertions::assert_eq;
    use uuid::uuid;

    use super::*;

    #[test]
    fn external_service_metadata_by_url_succeeds() {
        let external_service = ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
        };

        let actual = external_service.metadata_by_url("https://x.com/_namori_/status/727620202049900544").unwrap();
        assert_eq!(actual, ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) });
    }

    #[test]
    fn external_service_metadata_by_url_succeeds_invalid_url_pattern() {
        let external_service = ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"(".to_string()),
        };

        assert!(external_service.metadata_by_url("https://x.com/_namori_/status/727620202049900544").is_none());
    }

    #[test]
    fn external_service_metadata_by_url_succeeds_no_captures() {
        let external_service = ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/([^/]+)/status/(\d+)(?:[/?#].*)?$".to_string()),
        };

        assert!(external_service.metadata_by_url("https://www.pixiv.net/artworks/56736941").is_none());
    }

    #[test]
    fn external_service_metadata_by_url_succeeds_no_match() {
        let external_service = ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
        };

        assert!(external_service.metadata_by_url("https://www.pixiv.net/artworks/56736941").is_none());
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_bluesky() {
        let actual = ExternalMetadata::from_metadata("bluesky", "https://bsky.app/profile/creator_01/post/abcdefghi", Some("abcdefghi"), Some("creator_01")).unwrap();
        assert_eq!(actual, ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_fantia() {
        let actual = ExternalMetadata::from_metadata("fantia", "https://fantia.jp/posts/1305295", Some("1305295"), None).unwrap();
        assert_eq!(actual, ExternalMetadata::Fantia { id: 1305295 });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_mastodon() {
        let actual = ExternalMetadata::from_metadata("mastodon", "https://mastodon.social/@creator_01/123456789", Some("123456789"), Some("creator_01")).unwrap();
        assert_eq!(actual, ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_misskey() {
        let actual = ExternalMetadata::from_metadata("misskey", "https://misskey.io/notes/abcdefghi", Some("abcdefghi"), None).unwrap();
        assert_eq!(actual, ExternalMetadata::Misskey { id: "abcdefghi".to_string() });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_nijie() {
        let actual = ExternalMetadata::from_metadata("nijie", "https://nijie.info/view.php?id=323512", Some("323512"), None).unwrap();
        assert_eq!(actual, ExternalMetadata::Nijie { id: 323512 });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_pixiv() {
        let actual = ExternalMetadata::from_metadata("pixiv", "https://www.pixiv.net/artworks/56736941", Some("56736941"), None).unwrap();
        assert_eq!(actual, ExternalMetadata::Pixiv { id: 56736941 });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_pixiv_fanbox() {
        let actual = ExternalMetadata::from_metadata("pixiv_fanbox", "https://fairyeye.fanbox.cc/posts/178080", Some("178080"), Some("fairyeye")).unwrap();
        assert_eq!(actual, ExternalMetadata::PixivFanbox { id: 178080, creator_id: "fairyeye".to_string() });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_pleroma() {
        let actual = ExternalMetadata::from_metadata("pleroma", "https://udongein.xyz/notice/abcdefghi", Some("abcdefghi"), None).unwrap();
        assert_eq!(actual, ExternalMetadata::Pleroma { id: "abcdefghi".to_string() });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_seiga() {
        let actual = ExternalMetadata::from_metadata("seiga", "https://seiga.nicovideo.jp/seiga/6452903", Some("6452903"), None).unwrap();
        assert_eq!(actual, ExternalMetadata::Seiga { id: 6452903 });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_skeb() {
        let actual = ExternalMetadata::from_metadata("skeb", "https://skeb.jp/@pieleaf_x2/works/18", Some("18"), Some("pieleaf_x2")).unwrap();
        assert_eq!(actual, ExternalMetadata::Skeb { id: 18, creator_id: "pieleaf_x2".to_string() });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_threads() {
        let actual = ExternalMetadata::from_metadata("threads", "https://www.threads.net/creator_01/post/abcdefghi", Some("abcdefghi"), Some("creator_01")).unwrap();
        assert_eq!(actual, ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_website() {
        let actual = ExternalMetadata::from_metadata("website", "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885", None, None).unwrap();
        assert_eq!(actual, ExternalMetadata::Website { url: "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885".to_string() });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_x() {
        let actual = ExternalMetadata::from_metadata("x", "https://x.com/_namori_/status/727620202049900544", Some("727620202049900544"), Some("_namori_")).unwrap();
        assert_eq!(actual, ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) });
    }

    #[test]
    fn external_metadata_from_metadata_succeeds_with_xfolio() {
        let actual = ExternalMetadata::from_metadata("xfolio", "https://xfolio.jp/portfolio/creator_01/works/123456789", Some("123456789"), Some("creator_01")).unwrap();
        assert_eq!(actual, ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() });
    }

    #[test]
    fn external_metadata_kind_succeeds_with_bluesky() {
        let actual = ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() }.kind().unwrap();
        assert_eq!(actual, "bluesky");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_fantia() {
        let actual = ExternalMetadata::Fantia { id: 1305295 }.kind().unwrap();
        assert_eq!(actual, "fantia");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_mastodon() {
        let actual = ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() }.kind().unwrap();
        assert_eq!(actual, "mastodon");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_misskey() {
        let actual = ExternalMetadata::Misskey { id: "abcdefghi".to_string() }.kind().unwrap();
        assert_eq!(actual, "misskey");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_nijie() {
        let actual = ExternalMetadata::Nijie { id: 323512 }.kind().unwrap();
        assert_eq!(actual, "nijie");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_pixiv() {
        let actual = ExternalMetadata::Pixiv { id: 56736941 }.kind().unwrap();
        assert_eq!(actual, "pixiv");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_pixiv_fanbox() {
        let actual = ExternalMetadata::PixivFanbox { id: 178080, creator_id: "fairyeye".to_string() }.kind().unwrap();
        assert_eq!(actual, "pixiv_fanbox");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_pleroma() {
        let actual = ExternalMetadata::Pleroma { id: "abcdefghi".to_string() }.kind().unwrap();
        assert_eq!(actual, "pleroma");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_seiga() {
        let actual = ExternalMetadata::Seiga { id: 6452903 }.kind().unwrap();
        assert_eq!(actual, "seiga");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_skeb() {
        let actual = ExternalMetadata::Skeb { id: 18, creator_id: "pieleaf_x2".to_string() }.kind().unwrap();
        assert_eq!(actual, "skeb");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_threads() {
        let actual = ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) }.kind().unwrap();
        assert_eq!(actual, "threads");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_website() {
        let actual = ExternalMetadata::Website { url: "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885".to_string() }.kind().unwrap();
        assert_eq!(actual, "website");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_x() {
        let actual = ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) }.kind().unwrap();
        assert_eq!(actual, "x");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_xfolio() {
        let actual = ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() }.kind().unwrap();
        assert_eq!(actual, "xfolio");
    }

    #[test]
    fn external_metadata_kind_succeeds_with_custom() {
        let actual = ExternalMetadata::Custom(r#"{"id":42}"#.to_string()).kind();
        assert!(actual.is_none());
    }
}
