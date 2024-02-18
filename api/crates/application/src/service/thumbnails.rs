use std::{future::Future, sync::Arc};

use axum::{
    extract::{Path, State},
    response::Response,
};
use domain::entity::replicas::ThumbnailId;

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ThumbnailURLFactoryInterface: Send + Sync + 'static {
    fn get(&self, id: ThumbnailId) -> String;
}

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ThumbnailsServiceInterface: Send + Sync + 'static {
    fn show(&self, id: ThumbnailId) -> impl Future<Output = Response> + Send;
}

pub(crate) async fn show<ThumbnailsService>(thumbnails_service: State<Arc<ThumbnailsService>>, Path(id): Path<ThumbnailId>) -> Response
where
    ThumbnailsService: ThumbnailsServiceInterface,
{
    thumbnails_service.show(id).await
}
