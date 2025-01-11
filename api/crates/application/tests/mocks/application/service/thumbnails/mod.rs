use std::future::Future;

use application::service::thumbnails::ThumbnailsServiceInterface;
use axum::response::Response;
use domain::entity::replicas::ThumbnailId;

mockall::mock! {
    pub ThumbnailsServiceInterface {}

    impl ThumbnailsServiceInterface for ThumbnailsServiceInterface {
        fn show(&self, id: ThumbnailId) -> impl Future<Output = Response> + Send;
    }
}
