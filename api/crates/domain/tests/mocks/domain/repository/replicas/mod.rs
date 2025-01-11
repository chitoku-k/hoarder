use std::future::Future;

use domain::{
    entity::{
        media::MediumId,
        replicas::{OriginalImage, Replica, ReplicaId, ThumbnailId, ThumbnailImage},
    },
    error::Result,
    repository::{replicas::ReplicasRepository, DeleteResult},
};

mockall::mock! {
    pub ReplicasRepository {}

    impl ReplicasRepository for ReplicasRepository {
        fn create(&self, medium_id: MediumId, thumbnail_image: Option<ThumbnailImage>, original_url: &str, original_image: OriginalImage) -> impl Future<Output = Result<Replica>> + Send;

        fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Replica>>> + Send
        where
            T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

        fn fetch_by_original_url(&self, original_url: &str) -> impl Future<Output = Result<Replica>> + Send;

        fn fetch_thumbnail_by_id(&self, id: ThumbnailId) -> impl Future<Output = Result<Vec<u8>>> + Send;

        fn update_by_id<'a>(&self, id: ReplicaId, thumbnail_image: Option<ThumbnailImage>, original_url: Option<&'a str>, original_image: Option<OriginalImage>) -> impl Future<Output = Result<Replica>> + Send;

        fn delete_by_id(&self, id: ReplicaId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }
}
