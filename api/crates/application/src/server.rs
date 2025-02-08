use std::{net::Ipv6Addr, sync::Arc};

use axum::{extract::MatchedPath, http::Request, routing::{any, get, post}, Router};
use axum_server::Handle;
use tokio::{
    signal::unix::{self, SignalKind},
    task::JoinHandle,
};
use tower_http::trace::{DefaultOnEos, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[cfg(feature = "tls")]
use std::sync::mpsc::channel;
#[cfg(feature = "tls")]
use axum_server::tls_openssl::OpenSSLConfig;
#[cfg(feature = "tls")]
use notify::Watcher;

use crate::{
    error::{Error, ErrorKind, Result},
    service::{
        graphql::{self, GraphQLServiceInterface},
        objects::{self, ObjectsServiceInterface},
        thumbnails::{self, ThumbnailsServiceInterface},
    },
};

pub struct Engine {
    app: Router,
}

impl Engine {
    pub fn new<GraphQLService, ObjectsService, ThumbnailsService>(
        graphql_service: GraphQLService,
        objects_service: ObjectsService,
        thumbnails_service: ThumbnailsService,
    ) -> Self
    where
        GraphQLService: GraphQLServiceInterface,
        ObjectsService: ObjectsServiceInterface,
        ThumbnailsService: ThumbnailsServiceInterface,
    {
        let endpoints = graphql_service.endpoints();
        let graphql = Router::new()
            .route("/", get(graphql::graphiql::<GraphQLService>))
            .route(endpoints.graphql, post(graphql::execute::<GraphQLService>))
            .route(endpoints.subscriptions, any(graphql::subscriptions::<GraphQLService>))
            .with_state(Arc::new(graphql_service));

        let objects = Router::new()
            .route("/objects", get(objects::redirect::<ObjectsService>))
            .with_state(Arc::new(objects_service));

        let thumbnails = Router::new()
            .route("/thumbnails/{id}", get(thumbnails::show::<ThumbnailsService>))
            .with_state(Arc::new(thumbnails_service));

        let health = Router::new()
            .route("/healthz", get(|| async { "OK" }));

        let app = Router::new()
            .merge(graphql)
            .merge(objects)
            .merge(thumbnails)
            .layer(TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    let method = req.method();
                    let route = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|p| p.as_str());

                    tracing::debug_span!("request", ?method, route)
                })
                .on_request(DefaultOnRequest::new().level(Level::TRACE))
                .on_response(DefaultOnResponse::new().level(Level::TRACE))
                .on_eos(DefaultOnEos::new().level(Level::TRACE))
                .on_failure(DefaultOnFailure::new().level(Level::TRACE)))
            .merge(health);

        Self { app }
    }

    pub fn into_inner(self) -> Router {
        self.app
    }

    pub async fn start(self, port: u16, tls: Option<(String, String)>) -> Result<()> {
        let addr = (Ipv6Addr::UNSPECIFIED, port).into();

        let handle = Handle::new();
        enable_graceful_shutdown(handle.clone(), tls.is_some());

        match tls {
            #[cfg(not(feature = "tls"))]
            Some(_) => {
                panic!("TLS is not enabled.");
            },
            #[cfg(feature = "tls")]
            Some((tls_cert, tls_key)) => {
                let config = match OpenSSLConfig::from_pem_chain_file(&tls_cert, &tls_key) {
                    Ok(config) => {
                        enable_auto_reload(config.clone(), tls_cert, tls_key);
                        config
                    },
                    Err(e) => return Err(Error::new(ErrorKind::ServerCertificateInvalid { cert: tls_cert, key: tls_key }, e)),
                };
                axum_server::bind_openssl(addr, config)
                    .handle(handle)
                    .serve(self.app.into_make_service())
                    .await
                    .map_err(|e| Error::new(ErrorKind::ServerStartFailed, e))
            },
            None => {
                axum_server::bind(addr)
                    .handle(handle)
                    .serve(self.app.into_make_service())
                    .await
                    .map_err(|e| Error::new(ErrorKind::ServerStartFailed, e))
            },
        }
    }
}

#[cfg(feature = "tls")]
fn enable_auto_reload(config: OpenSSLConfig, tls_cert: String, tls_key: String) -> JoinHandle<Result<()>> {
    let (tx, rx) = channel();

    tokio::spawn(async move {
        let mut watcher = notify::recommended_watcher(tx).map_err(Error::other)?;
        watcher.watch(tls_cert.as_ref(), notify::RecursiveMode::NonRecursive).map_err(Error::other)?;

        for event in rx {
            let event = event.map_err(Error::other)?;
            if event.kind.is_modify() {
                if let Err(e) = config.reload_from_pem_file(&tls_cert, &tls_key) {
                    return Err(Error::new(ErrorKind::ServerCertificateInvalid { cert: tls_cert, key: tls_key }, e));
                }
            }
        }
        Ok(())
    })
}

fn enable_graceful_shutdown(handle: Handle, tls: bool) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let address = handle.listening().await.ok_or(ErrorKind::ServerBindFailed)?;
        let scheme = if tls { "https" } else { "http" };

        tracing::info!("listening on {scheme}://{address}/");

        let mut interrupt = unix::signal(SignalKind::interrupt()).map_err(Error::other)?;
        let mut terminate = unix::signal(SignalKind::terminate()).map_err(Error::other)?;

        tokio::select! {
            _ = interrupt.recv() => {},
            _ = terminate.recv() => {},
        };

        handle.graceful_shutdown(None);
        Ok(())
    })
}
