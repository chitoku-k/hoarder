use application::service::thumbnails::{ThumbnailsServiceInterface, ThumbnailURLFactoryInterface};
use axum::{
    body::Body,
    http::{
        header::CONTENT_TYPE,
        Response as HttpResponse,
        StatusCode,
    },
    response::{IntoResponse, Response},
};
use derive_more::Constructor;
use domain::{
    entity::replicas::ThumbnailId,
    error::ErrorKind,
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
    async fn show(&self, id: ThumbnailId) -> Response {
        match self.media_service.get_thumbnail_by_id(id).await {
            Ok(thumbnail) => {
                HttpResponse::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, "image/webp")
                    .body(Body::from(thumbnail))
                    .unwrap()
                    .into_response()
            },
            Err(e) if matches!(e.kind(), ErrorKind::ThumbnailNotFound { .. }) => {
                HttpResponse::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Not Found\n"))
                    .unwrap()
                    .into_response()
            },
            Err(_) => {
                HttpResponse::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Internal Server Error\n"))
                    .unwrap()
                    .into_response()
            },
        }
    }
}
