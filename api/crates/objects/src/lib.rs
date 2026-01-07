use std::sync::Arc;

use application::service::{
    media::MediaURLFactoryInterface,
    objects::ObjectsServiceInterface,
};
use axum::{
    body::Body,
    http::{
        Response as HttpResponse,
        StatusCode,
        header::{CONTENT_LENGTH, CONTENT_TYPE, ETAG, LAST_MODIFIED, LOCATION},
    },
    response::{IntoResponse, Response},
};
use derive_more::Constructor;
use domain::{
    entity::objects::EntryUrl,
    error::{Error, ErrorKind},
    service::media::MediaServiceInterface,
};
use tokio_util::io::ReaderStream;

mod http;
use crate::http::{HttpContentLength, HttpLastModified, HttpETag, ResponseBuilderExt};

#[derive(Clone, Constructor)]
pub struct ObjectsService<MediaService> {
    media_service: MediaService,
    media_url_factory: Arc<dyn MediaURLFactoryInterface>,
}

impl<MediaService> ObjectsServiceInterface for ObjectsService<MediaService>
where
    MediaService: MediaServiceInterface,
{
    #[tracing::instrument(skip_all)]
    async fn serve(&self, url: String) -> Response {
        enum Serve<Read> {
            Redirect(String),
            Content(Read, Option<HttpContentLength>, Option<HttpETag>, Option<HttpLastModified>),
            Error(Error),
        }

        let object = self.media_service
            .get_object(EntryUrl::from(url.clone()))
            .await
            .map(|entry| (entry.url, entry.metadata));

        let object = match object {
            Ok((Some(url), metadata)) => Ok((self.media_url_factory.public_url(&url), metadata)),
            Ok((None, metadata)) => Ok((None, metadata)),
            Err(e) => Err(e),
        };

        let serve = match object {
            Ok((Some(public_url), ..)) => Serve::Redirect(public_url),
            Ok((None, metadata)) => {
                let (content_length, etag, last_modified) = if let Some(metadata) = metadata {
                    let content_length = HttpContentLength(metadata.size);
                    let etag = HttpETag(metadata.size, metadata.updated_at);
                    let last_modified = metadata.updated_at.map(HttpLastModified);
                    (Some(content_length), Some(etag), last_modified)
                } else {
                    (None, None, None)
                };
                match self.media_service.read_object(EntryUrl::from(url)).await {
                    Ok(read) => Serve::Content(read, content_length, etag, last_modified),
                    Err(e) => Serve::Error(e),
                }
            },
            Err(e) => Serve::Error(e),
        };

        match serve {
            Serve::Redirect(public_url) => {
                HttpResponse::builder()
                    .status(StatusCode::FOUND)
                    .header(LOCATION, public_url)
                    .body(Body::from(()))
                    .unwrap()
                    .into_response()
            },
            Serve::Content(read, content_length, etag, last_modified) => {
                HttpResponse::builder()
                    .status(StatusCode::OK)
                    .header_opt(CONTENT_LENGTH, content_length)
                    .header_opt(ETAG, etag)
                    .header_opt(LAST_MODIFIED, last_modified)
                    .body(Body::from_stream(ReaderStream::new(read)))
                    .unwrap()
                    .into_response()
            },
            Serve::Error(e) if matches!(e.kind(), ErrorKind::ObjectPathInvalid) => {
                HttpResponse::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Bad Request: object path invalid\n"))
                    .unwrap()
                    .into_response()
            },
            Serve::Error(e) if matches!(e.kind(), ErrorKind::ObjectUrlInvalid { .. }) => {
                HttpResponse::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Bad Request: object url invalid\n"))
                    .unwrap()
                    .into_response()
            },
            Serve::Error(e) if matches!(e.kind(), ErrorKind::ObjectUrlUnsupported { .. }) => {
                HttpResponse::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Bad Request: object url unsupported\n"))
                    .unwrap()
                    .into_response()
            },
            Serve::Error(e) if matches!(e.kind(), ErrorKind::ObjectNotFound { .. }) => {
                HttpResponse::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Not Found\n"))
                    .unwrap()
                    .into_response()
            },
            Serve::Error(_) => {
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

#[cfg(test)]
mod tests;
