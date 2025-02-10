use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::entity::external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind};

#[test]
fn external_service_metadata_by_url_succeeds() {
    let external_service = ExternalService {
        id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        slug: "x".to_string(),
        kind: ExternalServiceKind::X,
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
        kind: ExternalServiceKind::X,
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
        kind: ExternalServiceKind::X,
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
        kind: ExternalServiceKind::X,
        name: "X".to_string(),
        base_url: Some("https://x.com".to_string()),
        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
    };

    assert!(external_service.metadata_by_url("https://www.pixiv.net/artworks/56736941").is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_bluesky() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Bluesky, "https://bsky.app/profile/creator_01/post/abcdefghi", Some("abcdefghi"), Some("creator_01")).unwrap();
    assert_eq!(actual, ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Bluesky, "https://bsky.app/profile/creator_01/post/abcdefghi", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Bluesky, "https://bsky.app/profile/creator_01/post/abcdefghi", None, Some("creator_01"));
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_fantia() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Fantia, "https://fantia.jp/posts/1305295", Some("1305295"), None).unwrap();
    assert_eq!(actual, ExternalMetadata::Fantia { id: 1305295 });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Fantia, "https://fantia.jp/posts/1305295", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Fantia, "https://fantia.jp/posts/1305295", None, None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_mastodon() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Mastodon, "https://mastodon.social/@creator_01/123456789", Some("123456789"), Some("creator_01")).unwrap();
    assert_eq!(actual, ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Mastodon, "https://mastodon.social/@creator_01/123456789", Some("123456789"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Mastodon, "https://mastodon.social/@creator_01/123456789", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Mastodon, "https://mastodon.social/@creator_01/123456789", None, Some("creator_01"));
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_misskey() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Misskey, "https://misskey.io/notes/abcdefghi", Some("abcdefghi"), None).unwrap();
    assert_eq!(actual, ExternalMetadata::Misskey { id: "abcdefghi".to_string() });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Misskey, "https://misskey.io/notes/abcdefghi", None, None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_nijie() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Nijie, "https://nijie.info/view.php?id=323512", Some("323512"), None).unwrap();
    assert_eq!(actual, ExternalMetadata::Nijie { id: 323512 });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Nijie, "https://nijie.info/view.php?id=323512", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Nijie, "https://nijie.info/view.php?id=323512", None, None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_pixiv() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Pixiv, "https://www.pixiv.net/artworks/56736941", Some("56736941"), None).unwrap();
    assert_eq!(actual, ExternalMetadata::Pixiv { id: 56736941 });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Pixiv, "https://www.pixiv.net/artworks/56736941", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Pixiv, "https://www.pixiv.net/artworks/56736941", None, None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_pixiv_fanbox() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::PixivFanbox, "https://fairyeye.fanbox.cc/posts/178080", Some("178080"), Some("fairyeye")).unwrap();
    assert_eq!(actual, ExternalMetadata::PixivFanbox { id: 178080, creator_id: "fairyeye".to_string() });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::PixivFanbox, "https://fairyeye.fanbox.cc/posts/178080", Some("178080"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::PixivFanbox, "https://fairyeye.fanbox.cc/posts/178080", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::PixivFanbox, "https://fairyeye.fanbox.cc/posts/178080", None, Some("fairyeye"));
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_pleroma() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Pleroma, "https://udongein.xyz/notice/abcdefghi", Some("abcdefghi"), None).unwrap();
    assert_eq!(actual, ExternalMetadata::Pleroma { id: "abcdefghi".to_string() });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Pleroma, "https://udongein.xyz/notice/abcdefghi", None, None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_seiga() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Seiga, "https://seiga.nicovideo.jp/seiga/6452903", Some("6452903"), None).unwrap();
    assert_eq!(actual, ExternalMetadata::Seiga { id: 6452903 });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Seiga, "https://seiga.nicovideo.jp/seiga/6452903", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Seiga, "https://seiga.nicovideo.jp/seiga/6452903", None, None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_skeb() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Skeb, "https://skeb.jp/@pieleaf_x2/works/18", Some("18"), Some("pieleaf_x2")).unwrap();
    assert_eq!(actual, ExternalMetadata::Skeb { id: 18, creator_id: "pieleaf_x2".to_string() });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Skeb, "https://skeb.jp/@pieleaf_x2/works/18", Some("18"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Skeb, "https://skeb.jp/@pieleaf_x2/works/18", Some("abcdefghi"), Some("pieleaf_x2"));
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Skeb, "https://skeb.jp/@pieleaf_x2/works/18", None, Some("pieleaf_x2"));
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_threads() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Threads, "https://www.threads.net/@creator_01/post/abcdefghi", Some("abcdefghi"), Some("creator_01")).unwrap();
    assert_eq!(actual, ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Threads, "https://www.threads.net/@creator_01/post/abcdefghi", Some("abcdefghi"), None).unwrap();
    assert_eq!(actual, ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: None });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Threads, "https://www.threads.net/@creator_01/post/abcdefghi", None, Some("creator_01"));
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_website() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Website, "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885", None, None).unwrap();
    assert_eq!(actual, ExternalMetadata::Website { url: "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885".to_string() });
}

#[test]
fn external_metadata_from_metadata_succeeds_with_x() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::X, "https://x.com/_namori_/status/727620202049900544", Some("727620202049900544"), Some("_namori_")).unwrap();
    assert_eq!(actual, ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::X, "https://x.com/_namori_/status/727620202049900544", Some("727620202049900544"), None).unwrap();
    assert_eq!(actual, ExternalMetadata::X { id: 727620202049900544, creator_id: None });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::X, "https://x.com/_namori_/status/727620202049900544", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::X, "https://x.com/_namori_/status/727620202049900544", None, None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_xfolio() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Xfolio, "https://xfolio.jp/portfolio/creator_01/works/123456789", Some("123456789"), Some("creator_01")).unwrap();
    assert_eq!(actual, ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() });

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Xfolio, "https://xfolio.jp/portfolio/creator_01/works/123456789", Some("123456789"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Xfolio, "https://xfolio.jp/portfolio/creator_01/works/123456789", Some("abcdefghi"), None);
    assert!(actual.is_none());

    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Xfolio, "https://xfolio.jp/portfolio/creator_01/works/123456789", None, Some("creator_01"));
    assert!(actual.is_none());
}

#[test]
fn external_metadata_from_metadata_succeeds_with_custom() {
    let actual = ExternalMetadata::from_metadata(&ExternalServiceKind::Custom("custom".to_string()), "https://example.com", None, None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_kind_succeeds_with_bluesky() {
    let actual = ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Bluesky);
}

#[test]
fn external_metadata_kind_succeeds_with_fantia() {
    let actual = ExternalMetadata::Fantia { id: 1305295 }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Fantia);
}

#[test]
fn external_metadata_kind_succeeds_with_mastodon() {
    let actual = ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Mastodon);
}

#[test]
fn external_metadata_kind_succeeds_with_misskey() {
    let actual = ExternalMetadata::Misskey { id: "abcdefghi".to_string() }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Misskey);
}

#[test]
fn external_metadata_kind_succeeds_with_nijie() {
    let actual = ExternalMetadata::Nijie { id: 323512 }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Nijie);
}

#[test]
fn external_metadata_kind_succeeds_with_pixiv() {
    let actual = ExternalMetadata::Pixiv { id: 56736941 }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Pixiv);
}

#[test]
fn external_metadata_kind_succeeds_with_pixiv_fanbox() {
    let actual = ExternalMetadata::PixivFanbox { id: 178080, creator_id: "fairyeye".to_string() }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::PixivFanbox);
}

#[test]
fn external_metadata_kind_succeeds_with_pleroma() {
    let actual = ExternalMetadata::Pleroma { id: "abcdefghi".to_string() }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Pleroma);
}

#[test]
fn external_metadata_kind_succeeds_with_seiga() {
    let actual = ExternalMetadata::Seiga { id: 6452903 }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Seiga);
}

#[test]
fn external_metadata_kind_succeeds_with_skeb() {
    let actual = ExternalMetadata::Skeb { id: 18, creator_id: "pieleaf_x2".to_string() }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Skeb);
}

#[test]
fn external_metadata_kind_succeeds_with_threads() {
    let actual = ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Threads);
}

#[test]
fn external_metadata_kind_succeeds_with_website() {
    let actual = ExternalMetadata::Website { url: "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885".to_string() }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Website);
}

#[test]
fn external_metadata_kind_succeeds_with_x() {
    let actual = ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::X);
}

#[test]
fn external_metadata_kind_succeeds_with_xfolio() {
    let actual = ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() }.kind().unwrap();
    assert_eq!(actual, ExternalServiceKind::Xfolio);
}

#[test]
fn external_metadata_kind_succeeds_with_custom() {
    let actual = ExternalMetadata::Custom(r#"{"id":42}"#.to_string()).kind();
    assert!(actual.is_none());
}

#[test]
fn external_metadata_url_succeeds_with_bluesky() {
    let actual = ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/profile/creator_01/post/abcdefghi");

    let actual = ExternalMetadata::Bluesky { id: "abcdefghi".to_string(), creator_id: "creator_01".to_string() }.url(None).unwrap();
    assert_eq!(actual, "https://bsky.app/profile/creator_01/post/abcdefghi");
}

#[test]
fn external_metadata_url_succeeds_with_fantia() {
    let actual = ExternalMetadata::Fantia { id: 1305295 }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/posts/1305295");

    let actual = ExternalMetadata::Fantia { id: 1305295 }.url(None).unwrap();
    assert_eq!(actual, "https://fantia.jp/posts/1305295");
}

#[test]
fn external_metadata_url_succeeds_with_mastodon() {
    let actual = ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() }.url(Some("https://mastodon.social/")).unwrap();
    assert_eq!(actual, "https://mastodon.social/@creator_01/123456789");

    let actual = ExternalMetadata::Mastodon { id: 123456789, creator_id: "creator_01".to_string() }.url(None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_url_succeeds_with_misskey() {
    let actual = ExternalMetadata::Misskey { id: "abcdefghi".to_string() }.url(Some("https://misskey.io/")).unwrap();
    assert_eq!(actual, "https://misskey.io/notes/abcdefghi");

    let actual = ExternalMetadata::Misskey { id: "abcdefghi".to_string() }.url(None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_url_succeeds_with_nijie() {
    let actual = ExternalMetadata::Nijie { id: 323512 }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/view.php?id=323512");

    let actual = ExternalMetadata::Nijie { id: 323512 }.url(None).unwrap();
    assert_eq!(actual, "https://nijie.info/view.php?id=323512");
}

#[test]
fn external_metadata_url_succeeds_with_pixiv() {
    let actual = ExternalMetadata::Pixiv { id: 56736941 }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/artworks/56736941");

    let actual = ExternalMetadata::Pixiv { id: 56736941 }.url(None).unwrap();
    assert_eq!(actual, "https://www.pixiv.net/artworks/56736941");
}

#[test]
fn external_metadata_url_succeeds_with_pixiv_fanbox() {
    let actual = ExternalMetadata::PixivFanbox { id: 178080, creator_id: "fairyeye".to_string() }.url(Some("https://example.com")).unwrap();
    assert_eq!(actual, "https://fairyeye.fanbox.cc/posts/178080");

    let actual = ExternalMetadata::PixivFanbox { id: 178080, creator_id: "fairyeye".to_string() }.url(None).unwrap();
    assert_eq!(actual, "https://fairyeye.fanbox.cc/posts/178080");
}

#[test]
fn external_metadata_url_succeeds_with_pleroma() {
    let actual = ExternalMetadata::Pleroma { id: "abcdefghi".to_string() }.url(Some("https://udongein.xyz")).unwrap();
    assert_eq!(actual, "https://udongein.xyz/notice/abcdefghi");

    let actual = ExternalMetadata::Pleroma { id: "abcdefghi".to_string() }.url(None);
    assert!(actual.is_none());
}

#[test]
fn external_metadata_url_succeeds_with_seiga() {
    let actual = ExternalMetadata::Seiga { id: 6452903 }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/seiga/im6452903");

    let actual = ExternalMetadata::Seiga { id: 6452903 }.url(None).unwrap();
    assert_eq!(actual, "https://seiga.nicovideo.jp/seiga/im6452903");
}

#[test]
fn external_metadata_url_succeeds_with_skeb() {
    let actual = ExternalMetadata::Skeb { id: 18, creator_id: "pieleaf_x2".to_string() }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/@pieleaf_x2/works/18");

    let actual = ExternalMetadata::Skeb { id: 18, creator_id: "pieleaf_x2".to_string() }.url(None).unwrap();
    assert_eq!(actual, "https://skeb.jp/@pieleaf_x2/works/18");
}

#[test]
fn external_metadata_url_succeeds_with_threads() {
    let actual = ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/@creator_01/post/abcdefghi");

    let actual = ExternalMetadata::Threads { id: "abcdefghi".to_string(), creator_id: Some("creator_01".to_string()) }.url(None).unwrap();
    assert_eq!(actual, "https://www.threads.net/@creator_01/post/abcdefghi");
}

#[test]
fn external_metadata_url_succeeds_with_website() {
    let actual = ExternalMetadata::Website { url: "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885".to_string() }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885");

    let actual = ExternalMetadata::Website { url: "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885".to_string() }.url(None).unwrap();
    assert_eq!(actual, "https://www.melonbooks.co.jp/corner/detail.php?corner_id=885");
}

#[test]
fn external_metadata_url_succeeds_with_x() {
    let actual = ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/_namori_/status/727620202049900544");

    let actual = ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) }.url(None).unwrap();
    assert_eq!(actual, "https://x.com/_namori_/status/727620202049900544");
}

#[test]
fn external_metadata_url_succeeds_with_xfolio() {
    let actual = ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() }.url(Some("https://example.com/")).unwrap();
    assert_eq!(actual, "https://example.com/portfolio/creator_01/works/123456789");

    let actual = ExternalMetadata::Xfolio { id: 123456789, creator_id: "creator_01".to_string() }.url(None).unwrap();
    assert_eq!(actual, "https://xfolio.jp/portfolio/creator_01/works/123456789");
}

#[test]
fn external_metadata_url_with_custom() {
    let actual = ExternalMetadata::Custom(r#"{"id":42}"#.to_string()).url(None);
    assert!(actual.is_none());
}
