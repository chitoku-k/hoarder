use std::future::Future;

use crate::{
    entity::external_services::{ExternalService, ExternalServiceId},
    error::Result,
    repository::DeleteResult
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ExternalServicesRepository: Send + Sync + 'static {
    /// Creates an external service.
    fn create<'a>(&self, slug: &str, kind: &str, name: &str, base_url: Option<&'a str>, url_pattern: Option<&'a str>) -> impl Future<Output = Result<ExternalService>> + Send;

    /// Fetches the external services by their IDs.
    fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<ExternalService>>> + Send
    where
        T: IntoIterator<Item = ExternalServiceId> + Send + Sync + 'static;

    /// Fetches all external services.
    fn fetch_all(&self) -> impl Future<Output = Result<Vec<ExternalService>>> + Send;

    /// Updates the external service by ID.
    fn update_by_id<'a>(&self, id: ExternalServiceId, slug: Option<&'a str>, name: Option<&'a str>, base_url: Option<Option<&'a str>>, url_pattern: Option<Option<&'a str>>) -> impl Future<Output = Result<ExternalService>> + Send;

    /// Deletes the external service by ID.
    fn delete_by_id(&self, id: ExternalServiceId) -> impl Future<Output = Result<DeleteResult>> + Send;
}
