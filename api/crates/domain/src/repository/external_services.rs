use async_trait::async_trait;

use crate::{
    entity::external_services::{ExternalService, ExternalServiceId},
    repository::DeleteResult,
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
#[async_trait]
pub trait ExternalServicesRepository: Send + Sync + 'static {
    /// Creates an external service.
    async fn create(&self, slug: &str, name: &str) -> anyhow::Result<ExternalService>;

    /// Fetches the external services by their IDs.
    async fn fetch_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<ExternalService>>
    where
        T: IntoIterator<Item = ExternalServiceId> + Send + Sync + 'static;

    /// Fetches all external services.
    async fn fetch_all(&self) -> anyhow::Result<Vec<ExternalService>>;

    /// Updates the external service by ID.
    async fn update_by_id<'a>(&self, id: ExternalServiceId, name: Option<&'a str>) -> anyhow::Result<ExternalService>;

    /// Deletes the external service by ID.
    async fn delete_by_id(&self, id: ExternalServiceId) -> anyhow::Result<DeleteResult>;
}
