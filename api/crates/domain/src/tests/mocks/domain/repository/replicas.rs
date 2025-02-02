use std::future::Future;

use crate::{
    entity::{
        media::MediumId,
        replicas::{OriginalImage, Replica, ReplicaId, ReplicaStatus, ThumbnailId, ThumbnailImage},
    },
    error::Result,
    iter::CloneableIterator,
    repository::{replicas::ReplicasRepository, DeleteResult},
};

mockall::mock! {
    pub(crate) ReplicasRepository {}

    impl ReplicasRepository for ReplicasRepository {
        fn create(&self, medium_id: MediumId, thumbnail_image: Option<ThumbnailImage>, original_url: &str, original_image: Option<OriginalImage>, status: ReplicaStatus) -> impl Future<Output = Result<Replica>> + Send;

        #[mockall::concretize]
        fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Replica>>> + Send
        where
            T: CloneableIterator<Item = ReplicaId> + Send;

        fn fetch_by_original_url(&self, original_url: &str) -> impl Future<Output = Result<Replica>> + Send;

        fn fetch_thumbnail_by_id(&self, id: ThumbnailId) -> impl Future<Output = Result<Vec<u8>>> + Send;

        fn update_by_id<'a>(&self, id: ReplicaId, thumbnail_image: Option<Option<ThumbnailImage>>, original_url: Option<&'a str>, original_image: Option<Option<OriginalImage>>, status: Option<ReplicaStatus>) -> impl Future<Output = Result<Replica>> + Send;

        fn delete_by_id(&self, id: ReplicaId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for ReplicasRepository {
        fn clone(&self) -> Self;
    }
}
