use std::future::Future;

use chrono::{DateTime, Utc};
use domain::{
    entity::{
        media::{Medium, MediumId},
        replicas::ReplicaId,
        sources::SourceId,
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    error::Result,
    repository::{media::MediaRepository, DeleteResult, Direction, Order},
};
use futures::stream::BoxStream;

mockall::mock! {
    pub MediaRepository {}

    impl MediaRepository for MediaRepository {
        fn create<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> impl Future<Output = Result<Medium>> + Send
        where
            T: IntoIterator<Item = SourceId> + Send + 'static,
            U: IntoIterator<Item = (TagId, TagTypeId)> + Send + 'static;

        fn fetch_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = Result<Vec<Medium>>> + Send
        where
            T: IntoIterator<Item = MediumId> + Send + 'static;

        fn fetch_by_source_ids<T>(
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
            T: IntoIterator<Item = SourceId> + Send + 'static;

        fn fetch_by_tag_ids<T>(
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
            T: IntoIterator<Item = (TagId, TagTypeId)> + Send + 'static;

        fn fetch_all(
            &self,
            tag_depth: Option<TagDepth>,
            replicas: bool,
            sources: bool,
            cursor: Option<(DateTime<Utc>, MediumId)>,
            order: Order,
            direction: Direction,
            limit: u64,
        ) -> impl Future<Output = Result<Vec<Medium>>> + Send;

        fn watch_by_id(&self, id: MediumId, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = Result<BoxStream<'static, Result<Medium>>>> + Send;

        fn update_by_id<T, U, V, W, X>(
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
            T: IntoIterator<Item = SourceId> + Send + 'static,
            U: IntoIterator<Item = SourceId> + Send + 'static,
            V: IntoIterator<Item = (TagId, TagTypeId)> + Send + 'static,
            W: IntoIterator<Item = (TagId, TagTypeId)> + Send + 'static,
            X: IntoIterator<Item = ReplicaId> + Send + 'static;

        fn delete_by_id(&self, id: MediumId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for MediaRepository {
        fn clone(&self) -> Self;
    }
}
