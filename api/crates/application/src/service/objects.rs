use std::sync::Arc;

use axum::{extract::{Query, State}, response::Response};
use serde::Deserialize;

pub trait ObjectsServiceInterface: Send + Sync + 'static {
    fn serve(&self, url: String) -> impl Future<Output = Response> + Send;
}

#[derive(Deserialize)]
pub(crate) struct GetParams {
    url: String,
}

pub(crate) async fn serve<ObjectsService>(objects_service: State<Arc<ObjectsService>>, Query(GetParams { url }): Query<GetParams>) -> Response
where
    ObjectsService: ObjectsServiceInterface,
{
    objects_service.serve(url).await
}
