use std::{io::stdout, time::Duration};

use application::{commands::PrintSchema, server::Engine};
use anyhow::Context;
use domain::service::{
    external_services::{ExternalServicesService, ExternalServicesServiceInterface},
    media::{MediaService, MediaServiceInterface},
    tags::{TagsService, TagsServiceInterface},
};
use graphql::{mutation::Mutation, query::Query, subscription::Subscription, APISchema};
use log::LevelFilter;
use postgres::{
    external_services::PostgresExternalServicesRepository,
    media::PostgresMediaRepository,
    replicas::PostgresReplicasRepository,
    sources::PostgresSourcesRepository,
    tag_types::PostgresTagTypesRepository,
    tags::PostgresTagsRepository,
    ConnectOptions, PgConnectOptions, PgPool, PgPoolOptions,
};
use thumbnails::{
    parser::WebPImageParser,
    ThumbnailURLFactory, ThumbnailsHandler,
};

use crate::env;

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

fn media_service<T, U, V, W>(media_repository: T, replicas_repository: U, sources_repository: V, thumbnail_image_parser: W) -> MediaService<T, U, V, W> {
    MediaService::new(media_repository, replicas_repository, sources_repository, thumbnail_image_parser)
}

fn tags_service<T, U>(tags_repository: T, tag_types_repository: U) -> TagsService<T, U> {
    TagsService::new(tags_repository, tag_types_repository)
}

fn query<T, U, V>(external_services_service: T, media_service: U, tags_service: V) -> Query<T, U, V> {
    Query::new(external_services_service, media_service, tags_service)
}

fn mutation<T, U, V>(external_services_service: T, media_service: U, tags_service: V) -> Mutation<T, U, V> {
    Mutation::new(external_services_service, media_service, tags_service)
}

fn schema<T, U, V>(query: Query<T, U, V>, mutation: Mutation<T, U, V>, thumbnail_url_factory: ThumbnailURLFactory) -> APISchema<T, U, V>
where
    T: ExternalServicesServiceInterface,
    U: MediaServiceInterface,
    V: TagsServiceInterface,
{
    APISchema::build(query, mutation, Subscription)
        .data(thumbnail_url_factory)
        .finish()
}

fn thumbnail_image_parser() -> WebPImageParser {
    WebPImageParser
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

        let config = env::get();

        let pg_options = PgConnectOptions::new()
            .log_statements(LevelFilter::Debug)
            .log_slow_statements(LevelFilter::Warn, Duration::from_millis(500));

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

        let thumbnail_image_parser = thumbnail_image_parser();

        let external_services_service = external_services_service(external_services_repository);
        let media_service = media_service(media_repository, replicas_repository, sources_repository, thumbnail_image_parser);
        let tags_service = tags_service(tags_repository, tag_types_repository);

        let thumbnail_url_factory = thumbnail_url_factory();
        let thumbnails_handler = thumbnails_handler(media_service.clone());

        let query = query(external_services_service.clone(), media_service.clone(), tags_service.clone());
        let mutation = mutation(external_services_service, media_service, tags_service);
        let schema = schema(query, mutation, thumbnail_url_factory);

        if config.print_schema {
            PrintSchema::new(schema).print(&mut stdout())?;
            return Ok(());
        }

        let tls = Option::zip(config.tls_cert, config.tls_key);
        let engine = Engine::new(
            config.port,
            tls,
            schema,
            thumbnails_handler,
        );

        engine.start().await
    }
}
