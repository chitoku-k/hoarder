use std::future::Future;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalServiceId},
        sources::{Source, SourceId},
    },
    error::Result,
    repository::DeleteResult,
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait SourcesRepository: Send + Sync + 'static {
    /// Creates a source.
    fn create(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Source>> + Send;

    /// Fetches the source by its external metadata.
    fn fetch_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Option<Source>>> + Send;

    /// Updates the source by ID.
    fn update_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> impl Future<Output = Result<Source>> + Send;

    /// Deletes the source by ID.
    fn delete_by_id(&self, id: SourceId) -> impl Future<Output = Result<DeleteResult>> + Send;
}
