use std::net::Ipv6Addr;

use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use axum_server::Handle;
use derive_more::Constructor;
use thiserror::Error;
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

use crate::service::{
    graphql::{self, GraphQLServiceInterface},
    thumbnails::{self, ThumbnailsServiceInterface},
};

#[derive(Constructor)]
pub struct Engine<GraphQLService, ThumbnailsService> {
    port: u16,
    tls: Option<(String, String)>,
    graphql_service: GraphQLService,
    thumbnails_service: ThumbnailsService,
}

#[derive(Debug, Error)]
pub(crate) enum EngineError {
    #[error("error starting server")]
    Serve,
    #[cfg(feature = "tls")]
    #[error("error loading certificate")]
    Certificate,
}

impl<GraphQLService, ThumbnailsService> Engine<GraphQLService, ThumbnailsService>
where
    GraphQLService: GraphQLServiceInterface + Clone,
    ThumbnailsService: ThumbnailsServiceInterface + Clone,
{
    pub async fn start(self) -> anyhow::Result<()> {
        let health = Router::new()
            .route("/", get(|| async { "OK" }));

        let graphql_endpoint = self.graphql_service.endpoint();
        let graphql = Router::new()
            .route(graphql_endpoint, post(graphql::execute::<GraphQLService>))
            .route("/", get(graphql::graphiql::<GraphQLService>))
            .with_state(self.graphql_service);

        let thumbnails = Router::new()
            .route("/:id", get(thumbnails::show::<ThumbnailsService>))
            .with_state(self.thumbnails_service);

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
