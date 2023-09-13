use async_trait::async_trait;
use axum::{
    body::BoxBody,
    extract::{Path, State},
    response::Response,
};
use domain::entity::replicas::ThumbnailId;

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ThumbnailURLFactoryInterface: Send + Sync + 'static {
    fn get(&self, id: ThumbnailId) -> String;
}

#[async_trait]
pub trait ThumbnailsServiceInterface: Send + Sync + 'static {
    async fn show(&self, id: ThumbnailId) -> Response<BoxBody>;
}

pub(crate) async fn show<ThumbnailsService>(thumbnails_service: State<ThumbnailsService>, Path(id): Path<ThumbnailId>) -> Response<BoxBody>
where
    ThumbnailsService: ThumbnailsServiceInterface,
{
    thumbnails_service.show(id).await
}
