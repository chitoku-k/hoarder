use std::future::Future;

use crate::{
    entity::external_services::{ExternalService, ExternalServiceId},
    repository::DeleteResult,
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ExternalServicesRepository: Send + Sync + 'static {
    /// Creates an external service.
    fn create(&self, slug: &str, name: &str) -> impl Future<Output = anyhow::Result<ExternalService>> + Send;

    /// Fetches the external services by their IDs.
    fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = anyhow::Result<Vec<ExternalService>>> + Send
    where
        T: IntoIterator<Item = ExternalServiceId> + Send + Sync + 'static;

    /// Fetches all external services.
    fn fetch_all(&self) -> impl Future<Output = anyhow::Result<Vec<ExternalService>>> + Send;

    /// Updates the external service by ID.
    fn update_by_id<'a>(&self, id: ExternalServiceId, name: Option<&'a str>) -> impl Future<Output = anyhow::Result<ExternalService>> + Send;

    /// Deletes the external service by ID.
    fn delete_by_id(&self, id: ExternalServiceId) -> impl Future<Output = anyhow::Result<DeleteResult>> + Send;
}
