use axum::response::Response;
use domain::entity::replicas::ThumbnailId;

use crate::service::thumbnails::ThumbnailsServiceInterface;

mockall::mock! {
    pub(crate) ThumbnailsServiceInterface {}

    impl ThumbnailsServiceInterface for ThumbnailsServiceInterface {
        fn show(&self, id: ThumbnailId) -> impl Future<Output = Response> + Send;
    }
}
