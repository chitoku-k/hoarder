use std::time::Duration;

use application::server::Engine;
use anyhow::Context;
use domain::service::{
    external_services::ExternalServicesService,
    media::MediaService,
    tags::TagsService,
};
use config::env;
use log::LevelFilter;
use postgres::{
    external_services::PostgresExternalServicesRepository,
    media::PostgresMediaRepository,
    replicas::PostgresReplicasRepository,
    sources::PostgresSourcesRepository,
    tag_types::PostgresTagTypesRepository,
    tags::PostgresTagsRepository,
};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, PgPool,
};
use thumbnails::{ThumbnailURLFactory, ThumbnailsHandler};

fn external_services_repository(pg_pool: PgPool) -> PostgresExternalServicesRepository {
    PostgresExternalServicesRepository::new(pg_pool)
}

fn media_repository(pg_pool: PgPool) -> PostgresMediaRepository {
    PostgresMediaRepository::new(pg_pool)
}

fn replicas_repository(pg_pool: PgPool) -> PostgresReplicasRepository {
    PostgresReplicasRepository::new(pg_pool)
}

fn sources_repository(pg_pool: PgPool) -> PostgresSourcesRepository {
    PostgresSourcesRepository::new(pg_pool)
}

fn tags_repository(pg_pool: PgPool) -> PostgresTagsRepository {
    PostgresTagsRepository::new(pg_pool)
}

fn tag_types_repository(pg_pool: PgPool) -> PostgresTagTypesRepository {
    PostgresTagTypesRepository::new(pg_pool)
}

fn external_services_service<T>(external_services_repository: T) -> ExternalServicesService<T> {
    ExternalServicesService::new(external_services_repository)
}

fn media_service<T, U, V>(media_repository: T, replicas_repository: U, sources_repository: V) -> MediaService<T, U, V> {
    MediaService::new(media_repository, replicas_repository, sources_repository)
}

fn tags_service<T, U>(tags_repository: T, tag_types_repository: U) -> TagsService<T, U> {
    TagsService::new(tags_repository, tag_types_repository)
}

fn thumbnail_url_factory() -> ThumbnailURLFactory {
    ThumbnailURLFactory::new("/thumbnails/".to_string())
}

fn thumbnails_handler<T>(media_service: T) -> ThumbnailsHandler<T> {
    ThumbnailsHandler::new(media_service)
}

pub struct Application;

impl Application {
    pub async fn start() -> anyhow::Result<()> {
        env_logger::builder()
            .format_target(true)
            .format_timestamp_secs()
            .format_indent(None)
            .filter(None, LevelFilter::Info)
            .parse_env("LOG_LEVEL")
            .init();

        let config = env::get()?;

        let mut pg_options = PgConnectOptions::new();
        pg_options.log_statements(LevelFilter::Debug);
        pg_options.log_slow_statements(LevelFilter::Warn, Duration::from_millis(500));

        let pg_pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(pg_options)
            .await
            .context("error connecting to database")?;

        let external_services_repository = external_services_repository(pg_pool.clone());
        let media_repository = media_repository(pg_pool.clone());
        let replicas_repository = replicas_repository(pg_pool.clone());
        let sources_repository = sources_repository(pg_pool.clone());
        let tags_repository = tags_repository(pg_pool.clone());
        let tag_types_repository = tag_types_repository(pg_pool);

        let external_services_service = external_services_service(external_services_repository);
        let media_service = media_service(media_repository, replicas_repository, sources_repository);
        let tags_service = tags_service(tags_repository, tag_types_repository);

        let thumbnail_url_factory = thumbnail_url_factory();
        let thumbnails_handler = thumbnails_handler(media_service.clone());

        let engine = Engine::new(
            config.port,
            Option::zip(config.tls_cert, config.tls_key),
            thumbnail_url_factory,
            thumbnails_handler,
            external_services_service,
            media_service,
            tags_service,
        );

        engine.start().await
    }
}
