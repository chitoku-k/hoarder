use async_graphql::SimpleObject;
use domain::entity::external_services;
use uuid::Uuid;

#[derive(SimpleObject)]
pub(crate) struct ExternalService {
    id: Uuid,
    slug: String,
    kind: String,
    name: String,
    base_url: Option<String>,
    url_pattern: Option<String>,
}

impl From<external_services::ExternalService> for ExternalService {
    fn from(external_service: external_services::ExternalService) -> Self {
        Self {
            id: *external_service.id,
            slug: external_service.slug,
            kind: external_service.kind,
            name: external_service.name,
            base_url: external_service.base_url,
            url_pattern: external_service.url_pattern,
        }
    }
}
