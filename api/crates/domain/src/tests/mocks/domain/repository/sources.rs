use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalServiceId},
        sources::{Source, SourceId},
    },
    error::Result,
    iter::CloneableIterator,
    repository::{sources::SourcesRepository, DeleteResult},
};

mockall::mock! {
    pub(crate) SourcesRepository {}

    impl SourcesRepository for SourcesRepository {
        fn create(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Source>> + Send;

        #[mockall::concretize]
        fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Source>>> + Send
        where
            T: CloneableIterator<Item = SourceId> + Send;

        fn fetch_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Option<Source>>> + Send;

        fn fetch_by_external_metadata_like_id(&self, id: &str) -> impl Future<Output = Result<Vec<Source>>> + Send;

        fn update_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> impl Future<Output = Result<Source>> + Send;

        fn delete_by_id(&self, id: SourceId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for SourcesRepository {
        fn clone(&self) -> Self;
    }
}
