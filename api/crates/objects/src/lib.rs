use std::sync::Arc;

use application::service::{
    media::MediaURLFactoryInterface,
    objects::ObjectsServiceInterface,
};
use axum::{
    body::Body,
    http::{
        header::{CONTENT_TYPE, LOCATION},
        Response as HttpResponse,
        StatusCode,
    },
    response::{IntoResponse, Response},
};
use derive_more::Constructor;
use domain::{
    entity::objects::EntryUrl,
    error::ErrorKind,
    service::media::MediaServiceInterface,
};

#[derive(Clone, Constructor)]
pub struct ObjectsService<MediaService> {
    media_service: MediaService,
    media_url_factory: Arc<dyn MediaURLFactoryInterface>,
}

impl<MediaService> ObjectsServiceInterface for ObjectsService<MediaService>
where
    MediaService: MediaServiceInterface,
{
    async fn redirect(&self, url: String) -> Response {
        let entry = self.media_service.get_object(&EntryUrl::from(url)).await;
        match entry.map(|entry| self.media_url_factory.public_url(&entry.url)) {
            Ok(Some(url)) => {
                HttpResponse::builder()
                    .status(StatusCode::FOUND)
                    .header(LOCATION, url)
                    .body(Body::from(()))
                    .unwrap()
                    .into_response()
            },
            Ok(None) => {
                HttpResponse::builder()
                    .status(StatusCode::FOUND)
                    .body(Body::from(()))
                    .unwrap()
                    .into_response()
            },
            Err(e) if matches!(e.kind(), ErrorKind::ObjectPathInvalid { .. } | ErrorKind::ObjectUrlUnsupported { .. }) => {
                HttpResponse::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Bad Request\n"))
                    .unwrap()
                    .into_response()
            },
            Err(e) if matches!(e.kind(), ErrorKind::ObjectNotFound { .. }) => {
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
