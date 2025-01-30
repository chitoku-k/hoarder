use application::service::thumbnails::ThumbnailURLFactoryInterface;
use domain::entity::replicas::ThumbnailId;

mockall::mock! {
    pub(crate) ThumbnailURLFactoryInterface {}

    impl ThumbnailURLFactoryInterface for ThumbnailURLFactoryInterface {
        fn get(&self, id: ThumbnailId) -> String;
    }
}
