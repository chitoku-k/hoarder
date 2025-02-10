use async_graphql::SimpleObject;
use domain::entity::external_services;
use uuid::Uuid;

/// An external service represents a Web service that is the origin of media.
#[derive(SimpleObject)]
pub(crate) struct ExternalService {
    /// The ID of the ExternalService object.
    id: Uuid,
    /// The short and user-friendly name that uniquely identifies the Web service.
    slug: String,
    /// The kind of the Web service that the object represents.
    /// Any other value than the following is considered a custom Web service:
    /// * `bluesky`: Bluesky
    /// * `fantia`: Fantia
    /// * `mastodon`: Mastodon
    /// * `misskey`: Misskey
    /// * `nijie`: ニジエ
    /// * `pixiv`: pixiv
    /// * `pixiv_fanbox`: pixivFANBOX
    /// * `pleroma`: Pleroma
    /// * `seiga`: ニコニコ静画
    /// * `skeb`: Skeb
    /// * `threads`: Threads
    /// * `x`: X
    /// * `xfolio`: Xfolio
    /// * `website`: any arbitrary website
    kind: String,
    /// The name of the Web service.
    name: String,
    /// The base URL of the Web service. Some services do not have the base URL.
    base_url: Option<String>,
    /// The regex pattern of a URL in the Web service.
    url_pattern: Option<String>,
}

impl From<external_services::ExternalService> for ExternalService {
    fn from(external_service: external_services::ExternalService) -> Self {
        Self {
            id: *external_service.id,
            slug: external_service.slug,
            kind: external_service.kind.to_string(),
            name: external_service.name,
            base_url: external_service.base_url,
            url_pattern: external_service.url_pattern,
        }
    }
}
