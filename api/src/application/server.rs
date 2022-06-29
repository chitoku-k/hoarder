use std::{
    net::Ipv6Addr,
    sync::{mpsc::channel, Arc},
    time::Duration,
};

use anyhow::Context;
use async_graphql::{EmptySubscription, Schema};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use axum_server::{tls_rustls::RustlsConfig, Handle};
use derive_more::Constructor;
use notify::{DebouncedEvent, Watcher};
use thiserror::Error;
use tokio::{
    signal::unix::{self, SignalKind},
    task::JoinHandle,
};

use crate::{
    application::{
        graphql::{self, mutation::Mutation, query::Query},
        thumbnails::{self, ThumbnailURLFactory, ThumbnailsHandler},
    },
    domain::service::{
        external_services::ExternalServicesServiceInterface,
        media::MediaServiceInterface,
        tags::TagsServiceInterface,
    },
};

#[derive(Clone, Constructor)]
pub struct Engine<ExternalServicesService, MediaService, TagsService> {
    port: u16,
    tls: Option<(String, String)>,
    thumbnail_url_factory: ThumbnailURLFactory,
    thumbnails_handler: ThumbnailsHandler<MediaService>,
    external_services_service: ExternalServicesService,
    media_service: MediaService,
    tags_service: TagsService,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("error starting server")]
    Serve,
    #[error("error loading certificate")]
    Certificate,
}

impl<ExternalServicesService, MediaService, TagsService> Engine<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface + Clone,
    MediaService: MediaServiceInterface + Clone,
    TagsService: TagsServiceInterface + Clone,
{
    pub async fn start(self) -> anyhow::Result<()> {
        let health = Router::new()
            .route("/", get(|| async { "OK" }));

        let query = Query::new(
            self.external_services_service.clone(),
            self.media_service.clone(),
            self.tags_service.clone(),
        );
        let mutation = Mutation::new(
            self.external_services_service,
            self.media_service,
            self.tags_service,
        );
        let schema = Schema::build(query, mutation, EmptySubscription)
            .data(self.thumbnail_url_factory)
            .finish();

        let graphql = Router::new()
            .route("/", post(graphql::handle::<ExternalServicesService, MediaService, TagsService>))
            .layer(Extension(schema));

        let graphql_playground = Router::new()
            .route("/", get(graphql::playground));

        let thumbnails = Router::new()
            .route("/:id", get(thumbnails::handle::<MediaService>))
            .layer(Extension(Arc::new(self.thumbnails_handler)));

        let handle = Handle::new();
        enable_graceful_shutdown(handle.clone());

        let addr = (Ipv6Addr::UNSPECIFIED, self.port).into();
        let app = Router::new()
            .nest("/", graphql)
            .nest("/", graphql_playground)
            .nest("/thumbnails", thumbnails)
            .nest("/healthz", health);

        match self.tls {
            Some((tls_cert, tls_key)) => {
                let config = RustlsConfig::from_pem_file(&tls_cert, &tls_key).await.context(EngineError::Certificate)?;
                enable_auto_reload(config.clone(), tls_cert, tls_key);

                axum_server::bind_rustls(addr, config)
                    .handle(handle)
                    .serve(app.into_make_service())
                    .await
                    .context(EngineError::Serve)
            },
            None => {
                axum_server::bind(addr)
                    .handle(handle)
                    .serve(app.into_make_service())
                    .await
                    .context(EngineError::Serve)
            },
        }
    }
}

fn enable_auto_reload(config: RustlsConfig, tls_cert: String, tls_key: String) -> JoinHandle<anyhow::Result<()>> {
    let (tx, rx) = channel();

    tokio::spawn(async move {
        let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
        watcher.watch(&tls_cert, notify::RecursiveMode::NonRecursive)?;

        for event in rx {
            if let DebouncedEvent::Write(_) = event {
                config.reload_from_pem_file(&tls_cert, &tls_key).await?;
            }
        }
        Ok(())
    })
}

fn enable_graceful_shutdown(handle: Handle) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        let mut stream = unix::signal(SignalKind::terminate())?;
        stream.recv().await;

        handle.graceful_shutdown(None);
        Ok(())
    })
}