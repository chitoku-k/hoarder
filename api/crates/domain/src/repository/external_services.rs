use std::future::Future;

use crate::{
    entity::external_services::{ExternalService, ExternalServiceId},
    error::Result,
    repository::DeleteResult
};

pub trait ExternalServicesRepository: Send + Sync + 'static {
    /// Creates an external service.
    fn create(&self, slug: &str, kind: &str, name: &str, base_url: Option<&str>, url_pattern: Option<&str>) -> impl Future<Output = Result<ExternalService>> + Send;

    /// Fetches the external services by their IDs.
    fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<ExternalService>>> + Send
    where
        for<'a> T: IntoIterator<Item = ExternalServiceId> + Send + 'a;

    /// Fetches all external services.
    fn fetch_all(&self) -> impl Future<Output = Result<Vec<ExternalService>>> + Send;

    /// Updates the external service by ID.
    fn update_by_id(&self, id: ExternalServiceId, slug: Option<&str>, name: Option<&str>, base_url: Option<Option<&str>>, url_pattern: Option<Option<&str>>) -> impl Future<Output = Result<ExternalService>> + Send;

    /// Deletes the external service by ID.
    fn delete_by_id(&self, id: ExternalServiceId) -> impl Future<Output = Result<DeleteResult>> + Send;
}
