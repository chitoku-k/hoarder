use std::{sync::Arc, time::Duration};

use application::{
    server::Engine,
    service::{
        graphql::GraphQLServiceInterface,
        media::MediaURLFactoryInterface,
        thumbnails::ThumbnailURLFactoryInterface,
    },
};
use anyhow::Context;
use domain::{
    entity::replicas::Size,
    service::{
        external_services::{ExternalServicesService, ExternalServicesServiceInterface},
        media::{MediaService, MediaServiceInterface},
        tags::{TagsService, TagsServiceInterface},
    },
};
use graphql::{mutation::Mutation, query::Query, subscription::Subscription, APISchema, GraphQLService};
use icu::collator::{Collator, CollatorOptions};
use icu_provider::DataLocale;
use log::LevelFilter;
use media::{FileMediaURLFactory, NoopMediaURLFactory};
use objects::ObjectsService;
use postgres::{
    external_services::PostgresExternalServicesRepository,
    media::PostgresMediaRepository,
    replicas::PostgresReplicasRepository,
    sources::PostgresSourcesRepository,
    tag_types::PostgresTagTypesRepository,
    tags::PostgresTagsRepository,
    ConnectOptions, PgConnectOptions, PgPool, PgPoolOptions,
};
use storage::filesystem::FilesystemObjectsRepository;
use thumbnails::{
    processor::{FilterType, ImageFormat, InMemoryImageProcessor},
    ThumbnailURLFactory, ThumbnailsService,
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

fn objects_repository(collator: Collator, root_dir: String) -> FilesystemObjectsRepository {
    FilesystemObjectsRepository::new(Arc::new(collator), root_dir)
}

fn external_services_service<T>(external_services_repository: T) -> ExternalServicesService<T> {
    ExternalServicesService::new(external_services_repository)
}

fn media_service<T, U, V, W, X>(media_repository: T, objects_repository: U, replicas_repository: V, sources_repository: W, medium_image_processor: X) -> MediaService<T, U, V, W, X> {
    MediaService::new(media_repository, objects_repository, replicas_repository, sources_repository, medium_image_processor)
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

fn schema<T, U, V>(
    query: Query<T, U, V>,
    mutation: Mutation<T, U, V>,
    media_url_factory: Arc<dyn MediaURLFactoryInterface>,
    thumbnail_url_factory: Arc<dyn ThumbnailURLFactoryInterface>,
) -> APISchema<T, U, V>
where
    T: ExternalServicesServiceInterface,
    U: MediaServiceInterface,
    V: TagsServiceInterface,
{
    APISchema::build(query, mutation, Subscription)
        .data(media_url_factory)
        .data(thumbnail_url_factory)
        .finish()
}

fn medium_image_processor() -> InMemoryImageProcessor {
    InMemoryImageProcessor::new(Size::new(240, 240), ImageFormat::WebP, FilterType::CatmullRom)
}

fn graphql_service<T, U, V>(schema: APISchema<T, U, V>) -> GraphQLService<T, U, V>
where
    T: ExternalServicesServiceInterface,
    U: MediaServiceInterface,
    V: TagsServiceInterface,
{
    GraphQLService::new(schema, "/graphql")
}

fn file_media_url_factory(root_url: String) -> FileMediaURLFactory {
    FileMediaURLFactory::new(root_url)
}

fn noop_media_url_factory() -> NoopMediaURLFactory {
    NoopMediaURLFactory::new()
}

fn objects_service<T>(media_service: T, media_url_factory: Arc<dyn MediaURLFactoryInterface>) -> ObjectsService<T>
where
    T: MediaServiceInterface,
{
    ObjectsService::new(media_service, media_url_factory)
}

fn thumbnail_url_factory() -> ThumbnailURLFactory {
    ThumbnailURLFactory::new("/thumbnails")
}

fn thumbnails_service<T>(media_service: T) -> ThumbnailsService<T> {
    ThumbnailsService::new(media_service)
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
        let collator = Collator::try_new(&DataLocale::from(config.locale), CollatorOptions::new())
            .context("error instantiating collator")?;

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

        let objects_repository = objects_repository(collator, config.media_root_dir);
        let medium_image_processor = medium_image_processor();

        let external_services_service = external_services_service(external_services_repository);
        let media_service = media_service(media_repository, objects_repository, replicas_repository, sources_repository, medium_image_processor);
        let tags_service = tags_service(tags_repository, tag_types_repository);

        let media_url_factory: Arc<dyn MediaURLFactoryInterface> = match config.media_root_url {
            Some(media_root_url) => Arc::new(file_media_url_factory(media_root_url)),
            None => Arc::new(noop_media_url_factory()),
        };

        let objects_service = objects_service(media_service.clone(), media_url_factory.clone());

        let thumbnail_url_factory = Arc::new(thumbnail_url_factory());
        let thumbnails_service = thumbnails_service(media_service.clone());

        let query = query(external_services_service.clone(), media_service.clone(), tags_service.clone());
        let mutation = mutation(external_services_service, media_service, tags_service);
        let schema = schema(query, mutation, media_url_factory, thumbnail_url_factory);
        let graphql_service = graphql_service(schema);

        if config.print_schema {
            println!("{}", graphql_service.definitions());
            return Ok(());
        }

        let tls = Option::zip(config.tls_cert, config.tls_key);
        Engine::new(graphql_service, objects_service, thumbnails_service)
            .start(config.port, tls)
            .await?;

        Ok(())
    }
}
