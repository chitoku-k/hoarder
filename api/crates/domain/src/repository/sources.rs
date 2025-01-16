use std::future::Future;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalServiceId},
        sources::{Source, SourceId},
    },
    error::Result,
    repository::DeleteResult,
};

pub trait SourcesRepository: Send + Sync + 'static {
    /// Creates a source.
    fn create(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Source>> + Send;

    /// Fetches the sources by their IDs.
    fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Source>>> + Send
    where
        for<'a> T: IntoIterator<Item = SourceId> + Send + 'a;

    /// Fetches the source by its external metadata.
    fn fetch_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Option<Source>>> + Send;

    /// Fetches the sources by ID field of their external metadata.
    fn fetch_by_external_metadata_like_id(&self, id: &str) -> impl Future<Output = Result<Vec<Source>>> + Send;

    /// Updates the source by ID.
    fn update_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> impl Future<Output = Result<Source>> + Send;

    /// Deletes the source by ID.
    fn delete_by_id(&self, id: SourceId) -> impl Future<Output = Result<DeleteResult>> + Send;
}
