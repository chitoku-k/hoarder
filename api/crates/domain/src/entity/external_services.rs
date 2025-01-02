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
    pub url_pattern: Option<String>,
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
