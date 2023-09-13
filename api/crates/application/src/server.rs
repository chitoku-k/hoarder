use std::{net::Ipv6Addr, sync::Arc};

use anyhow::Context;
use axum::{
    routing::{get, post},
    Extension, Router,
};
use axum_server::Handle;
use derive_more::Constructor;
use domain::service::{
    external_services::ExternalServicesServiceInterface,
    media::MediaServiceInterface,
    tags::TagsServiceInterface,
};
use graphql::{self, APISchema};
use thiserror::Error;
use thumbnails::{self, ThumbnailsHandler};
use tokio::{
    signal::unix::{self, SignalKind},
    task::JoinHandle,
};

#[cfg(feature = "tls")]
use std::sync::mpsc::channel;
#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
use notify::Watcher;

#[derive(Constructor)]
pub struct Engine<ExternalServicesService, MediaService, TagsService> {
    port: u16,
    tls: Option<(String, String)>,
    schema: APISchema<ExternalServicesService, MediaService, TagsService>,
    thumbnails_handler: ThumbnailsHandler<MediaService>,
}

#[derive(Debug, Error)]
pub(crate) enum EngineError {
    #[error("error starting server")]
    Serve,
    #[cfg(feature = "tls")]
    #[error("error loading certificate")]
    Certificate,
}

impl<ExternalServicesService, MediaService, TagsService> Engine<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
{
    pub async fn start(self) -> anyhow::Result<()> {
        let health = Router::new()
            .route("/", get(|| async { "OK" }));

        let graphql = Router::new()
            .route("/graphql", post(graphql::handle::<ExternalServicesService, MediaService, TagsService>))
            .route("/", get(graphql::graphiql))
            .layer(Extension(self.schema));

        let thumbnails = Router::new()
            .route("/:id", get(thumbnails::handle::<MediaService>))
            .layer(Extension(Arc::new(self.thumbnails_handler)));

        let handle = Handle::new();
        enable_graceful_shutdown(handle.clone(), self.tls.is_some());

        let addr = (Ipv6Addr::UNSPECIFIED, self.port).into();
        let app = Router::new()
            .nest("/", graphql)
            .nest("/thumbnails", thumbnails)
            .nest("/healthz", health);

        match self.tls {
            #[cfg(not(feature = "tls"))]
            Some(_) => {
                panic!("TLS is not enabled.");
            },
            #[cfg(feature = "tls")]
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

#[cfg(feature = "tls")]
fn enable_auto_reload(config: RustlsConfig, tls_cert: String, tls_key: String) -> JoinHandle<anyhow::Result<()>> {
    let (tx, rx) = channel();

    tokio::spawn(async move {
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(tls_cert.as_ref(), notify::RecursiveMode::NonRecursive)?;

        for event in rx {
            if event?.kind.is_modify() {
                config.reload_from_pem_file(&tls_cert, &tls_key).await?;
            }
        }
        Ok(())
    })
}

fn enable_graceful_shutdown(handle: Handle, tls: bool) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        let address = handle.listening().await.context("failed to bind")?;
        let scheme = if tls { "https" } else { "http" };

        log::info!("listening on {scheme}://{address}/");

        let mut stream = unix::signal(SignalKind::terminate())?;
        stream.recv().await;

        handle.graceful_shutdown(None);
        Ok(())
    })
}
