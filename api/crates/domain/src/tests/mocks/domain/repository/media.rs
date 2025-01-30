use std::future::Future;

use chrono::{DateTime, Utc};
use futures::stream::BoxStream;

use crate::{
    entity::{
        media::{Medium, MediumId},
        replicas::ReplicaId,
        sources::SourceId,
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    error::Result,
    iter::CloneableIterator,
    repository::{media::MediaRepository, DeleteResult, Direction, Order},
};

mockall::mock! {
    pub(crate) MediaRepository {}

    impl MediaRepository for MediaRepository {
        #[mockall::concretize]
        fn create<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> impl Future<Output = Result<Medium>> + Send
        where
            T: CloneableIterator<Item = SourceId> + Send,
            U: CloneableIterator<Item = (TagId, TagTypeId)> + Send;

        #[mockall::concretize]
        fn fetch_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = Result<Vec<Medium>>> + Send
        where
            T: CloneableIterator<Item = MediumId> + Send;

        #[mockall::concretize]
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
            T: CloneableIterator<Item = SourceId> + Send;

        #[mockall::concretize]
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
            T: CloneableIterator<Item = (TagId, TagTypeId)> + Send;

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

        #[mockall::concretize]
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
            T: CloneableIterator<Item = SourceId> + Send,
            U: CloneableIterator<Item = SourceId> + Send,
            V: CloneableIterator<Item = (TagId, TagTypeId)> + Send,
            W: CloneableIterator<Item = (TagId, TagTypeId)> + Send,
            X: CloneableIterator<Item = ReplicaId> + Send;

        fn delete_by_id(&self, id: MediumId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for MediaRepository {
        fn clone(&self) -> Self;
    }
}
