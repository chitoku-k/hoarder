use async_graphql::SimpleObject;
use uuid::Uuid;

use crate::domain::entity::external_services;

#[derive(SimpleObject)]
pub struct ExternalService {
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
