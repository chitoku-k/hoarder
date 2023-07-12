use async_graphql::SimpleObject;
use domain::entity::external_services;
use uuid::Uuid;

#[derive(SimpleObject)]
pub(crate) struct ExternalService {
    id: Uuid,
    slug: String,
    name: String,
}

impl From<external_services::ExternalService> for ExternalService {
    fn from(external_service: external_services::ExternalService) -> Self {
        Self {
            id: *external_service.id,
            slug: external_service.slug,
            name: external_service.name,
        }
    }
}
