use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
};
use domain::entity::replicas::ThumbnailId;

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ThumbnailURLFactoryInterface: Send + Sync + 'static {
    fn get(&self, id: ThumbnailId) -> String;
}

#[cfg_attr(feature = "test-mock", mockall::automock)]
#[async_trait]
pub trait ThumbnailsServiceInterface: Send + Sync + 'static {
    async fn show(&self, id: ThumbnailId) -> Response<Body>;
}

pub(crate) async fn show<ThumbnailsService>(thumbnails_service: State<Arc<ThumbnailsService>>, Path(id): Path<ThumbnailId>) -> Response<Body>
where
    ThumbnailsService: ThumbnailsServiceInterface,
{
    thumbnails_service.show(id).await
}
