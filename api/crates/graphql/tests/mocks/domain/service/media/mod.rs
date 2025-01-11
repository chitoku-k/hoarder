use std::future::Future;

use chrono::{DateTime, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalServiceId},
        media::{Medium, MediumId},
        objects::{Entry, EntryKind, EntryUrl, EntryUrlPath},
        replicas::{Replica, ReplicaId, ThumbnailId},
        sources::{Source, SourceId},
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    error::Result,
    repository::{DeleteResult, Direction, Order},
    service::media::{MediaServiceInterface, MediumSource},
};

mockall::mock! {
    pub MediaServiceInterface {}

    impl MediaServiceInterface for MediaServiceInterface {
        fn create_medium<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> impl Future<Output = Result<Medium>> + Send
        where
            T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
            U: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static;

        fn create_replica(&self, medium_id: MediumId, medium_source: MediumSource) -> impl Future<Output = Result<Replica>> + Send;

        fn create_source(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Source>> + Send;

        fn get_media(
            &self,
            tag_depth: Option<TagDepth>,
            replicas: bool,
            sources: bool,
            cursor: Option<(DateTime<Utc>, MediumId)>,
            order: Order,
            direction: Direction,
            limit: u64,
        ) -> impl Future<Output = Result<Vec<Medium>>> + Send;

        fn get_media_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = Result<Vec<Medium>>> + Send
        where
            T: IntoIterator<Item = MediumId> + Send + Sync + 'static;

        fn get_media_by_source_ids<T>(
            &self,
            source_ids: T,
            tag_depth: Option<TagDepth>,
            replicas: bool,
            sources: bool,
            cursor: Option<(DateTime<Utc>, MediumId)>,
            order: Order,
            direction: Direction,
            limit: u64,
        ) -> impl Future<Output = Result<Vec<Medium>>> + Send
        where
            T: IntoIterator<Item = SourceId> + Send + Sync + 'static;

        fn get_media_by_tag_ids<T>(
            &self,
            tag_tag_type_ids: T,
            tag_depth: Option<TagDepth>,
            replicas: bool,
            sources: bool,
            cursor: Option<(DateTime<Utc>, MediumId)>,
            order: Order,
            direction: Direction,
            limit: u64,
        ) -> impl Future<Output = Result<Vec<Medium>>> + Send
        where
            T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static;

        fn get_replicas_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Replica>>> + Send
        where
            T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

        fn get_replica_by_original_url(&self, original_url: &str) -> impl Future<Output = Result<Replica>> + Send;

        fn get_sources_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Source>>> + Send
        where
            T: IntoIterator<Item = SourceId> + Send + Sync + 'static;

        fn get_source_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Option<Source>>> + Send;

        fn get_sources_by_external_metadata_like_id(&self, id: &str) -> impl Future<Output = Result<Vec<Source>>> + Send;

        fn get_thumbnail_by_id(&self, id: ThumbnailId) -> impl Future<Output = Result<Vec<u8>>> + Send;

        fn get_object(&self, url: EntryUrl) -> impl Future<Output = Result<Entry>> + Send;

        fn get_objects(&self, prefix: EntryUrlPath, kind: Option<EntryKind>) -> impl Future<Output = Result<Vec<Entry>>> + Send;

        fn update_medium_by_id<T, U, V, W, X>(
            &self,
            id: MediumId,
            add_source_ids: T,
            remove_source_ids: U,
            add_tag_tag_type_ids: V,
            remove_tag_tag_type_ids: W,
            replica_orders: X,
            created_at: Option<DateTime<Utc>>,
            tag_depth: Option<TagDepth>,
            replicas: bool,
            sources: bool,
        ) -> impl Future<Output = Result<Medium>> + Send
        where
            T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
            U: IntoIterator<Item = SourceId> + Send + Sync + 'static,
            V: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
            W: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
            X: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

        fn update_replica_by_id(&self, id: ReplicaId, medium_source: MediumSource) -> impl Future<Output = Result<Replica>> + Send;

        fn update_source_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> impl Future<Output = Result<Source>> + Send;

        fn delete_medium_by_id(&self, id: MediumId, delete_objects: bool) -> impl Future<Output = Result<DeleteResult>> + Send;

        fn delete_replica_by_id(&self, id: ReplicaId, delete_object: bool) -> impl Future<Output = Result<DeleteResult>> + Send;

        fn delete_source_by_id(&self, id: SourceId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }
}
