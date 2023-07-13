use async_trait::async_trait;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalServiceId},
        sources::{Source, SourceId},
    },
    repository::DeleteResult,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SourcesRepository: Send + Sync + 'static {
    /// Creates a source.
    async fn create(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Source>;

    /// Fetches the source by its external metadata.
    async fn fetch_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Source>;

    /// Updates the source by ID.
    async fn update_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> anyhow::Result<Source>;

    /// Deletes the source by ID.
    async fn delete_by_id(&self, id: SourceId) -> anyhow::Result<DeleteResult>;
}
