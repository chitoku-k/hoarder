use std::{future::Future, sync::Arc};

use axum::{extract::{Query, State}, response::Response};
use serde::Deserialize;

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ObjectsServiceInterface: Send + Sync + 'static {
    fn redirect(&self, url: String) -> impl Future<Output = Response> + Send;
}

#[derive(Deserialize)]
pub(crate) struct GetParams {
    url: String,
}

pub(crate) async fn redirect<ObjectsService>(objects_service: State<Arc<ObjectsService>>, Query(GetParams { url }): Query<GetParams>) -> Response
where
    ObjectsService: ObjectsServiceInterface,
{
    objects_service.redirect(url).await
}
