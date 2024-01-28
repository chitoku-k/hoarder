use application::service::thumbnails::{ThumbnailsServiceInterface, ThumbnailURLFactoryInterface};
use axum::{
    body::Body,
    http::{
        Response as HttpResponse,
        StatusCode,
    },
    response::{Response, IntoResponse},
};
use derive_more::Constructor;
use domain::{
    entity::replicas::ThumbnailId,
    service::media::MediaServiceInterface,
};

pub mod processor;

#[derive(Constructor)]
pub struct ThumbnailURLFactory {
    endpoint: &'static str,
}

impl ThumbnailURLFactoryInterface for ThumbnailURLFactory {
    fn get(&self, id: ThumbnailId) -> String {
        format!("{}/{}", self.endpoint, id)
    }
}

#[derive(Clone, Constructor)]
pub struct ThumbnailsService<MediaService> {
    media_service: MediaService,
}

impl<MediaService> ThumbnailsServiceInterface for ThumbnailsService<MediaService>
where
    MediaService: MediaServiceInterface,
{
    async fn show(&self, id: ThumbnailId) -> Response<Body> {
        match self.media_service.get_thumbnail_by_id(id).await {
            Ok(thumbnail) => {
                HttpResponse::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "image/webp")
                    .body(Body::from(thumbnail))
                    .unwrap()
                    .into_response()
            },
            Err(_) => {
                HttpResponse::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(Body::from("Not Found\n"))
                    .unwrap()
                    .into_response()
            },
        }
    }
}
