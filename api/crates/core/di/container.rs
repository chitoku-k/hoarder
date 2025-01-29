use std::{sync::Arc, time::Duration};

use application::{server::Engine, service::{graphql::GraphQLServiceInterface, media::MediaURLFactoryInterface}};
use anyhow::Context;
use domain::{
    entity::replicas::Size,
    service::{
        external_services::ExternalServicesService,
        media::MediaService,
        tags::TagsService,
    },
};
use graphql::{mutation::Mutation, query::Query, subscription::Subscription, Schema, SchemaBuilder, GraphQLService};
use icu_collator::{Collator, CollatorOptions};
use icu_provider::DataLocale;
use log::LevelFilter;
use media::{FileMediaURLFactory, NoopMediaURLFactory};
use normalizer::Normalizer;
use objects::ObjectsService;
use postgres::{
    external_services::PostgresExternalServicesRepository,
    media::PostgresMediaRepository,
    replicas::PostgresReplicasRepository,
    sources::PostgresSourcesRepository,
    tag_types::PostgresTagTypesRepository,
    tags::PostgresTagsRepository,
    ConnectOptions, Migrator, PgConnectOptions, PgPool, PgPoolOptions,
};
use storage::filesystem::FilesystemObjectsRepository;
use thumbnails::{
    processor::{FilterType, ImageFormat, InMemoryImageProcessor},
    ThumbnailURLFactory, ThumbnailsService,
};
use tokio_util::task::TaskTracker;

use crate::env::{self, commands::{Commands, SchemaCommand, SchemaCommands}};

type ExternalServicesRepositoryImpl = PostgresExternalServicesRepository;
type MediaRepositoryImpl = PostgresMediaRepository;
type ReplicasRepositoryImpl = PostgresReplicasRepository;
type SourcesRepositoryImpl = PostgresSourcesRepository;
type TagsRepositoryImpl = PostgresTagsRepository;
type TagTypesRepositoryImpl = PostgresTagTypesRepository;
type ObjectsRepositoryImpl = FilesystemObjectsRepository;
type NormalizerImpl = Normalizer;
type ExternalServicesServiceImpl = ExternalServicesService<ExternalServicesRepositoryImpl>;
type MediaServiceImpl = MediaService<MediaRepositoryImpl, ObjectsRepositoryImpl, ReplicasRepositoryImpl, SourcesRepositoryImpl, MediumImageProcessorImpl>;
type TagsServiceImpl = TagsService<TagsRepositoryImpl, TagTypesRepositoryImpl>;
type ObjectsServiceImpl = ObjectsService<MediaServiceImpl>;
type ThumbnailsServiceImpl = ThumbnailsService<MediaServiceImpl>;
type QueryImpl = Query<ExternalServicesServiceImpl, MediaServiceImpl, TagsServiceImpl, NormalizerImpl>;
type MutationImpl = Mutation<ExternalServicesServiceImpl, MediaServiceImpl, TagsServiceImpl, NormalizerImpl>;
type SubscriptionImpl = Subscription<MediaServiceImpl>;
type SchemaImpl = Schema<QueryImpl, MutationImpl, SubscriptionImpl>;
type SchemaBuilderImpl = SchemaBuilder<QueryImpl, MutationImpl, SubscriptionImpl>;
type MediumImageProcessorImpl = InMemoryImageProcessor;
type GraphQLServiceImpl = GraphQLService<QueryImpl, MutationImpl, SubscriptionImpl>;

async fn pg_pool() -> anyhow::Result<PgPool> {
    let pg_options = PgConnectOptions::new()
        .log_statements(LevelFilter::Debug)
        .log_slow_statements(LevelFilter::Warn, Duration::from_millis(500));

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(pg_options)
        .await
        .context("error connecting to database")?;

    Ok(pg_pool)
}

fn task_tracker() -> TaskTracker {
    TaskTracker::new()
}

fn external_services_repository(pg_pool: PgPool) -> ExternalServicesRepositoryImpl {
    PostgresExternalServicesRepository::new(pg_pool)
}

fn media_repository(pg_pool: PgPool) -> MediaRepositoryImpl {
    PostgresMediaRepository::new(pg_pool)
}

fn replicas_repository(pg_pool: PgPool) -> ReplicasRepositoryImpl {
    PostgresReplicasRepository::new(pg_pool)
}

fn sources_repository(pg_pool: PgPool) -> SourcesRepositoryImpl {
    PostgresSourcesRepository::new(pg_pool)
}

fn tags_repository(pg_pool: PgPool) -> TagsRepositoryImpl {
    PostgresTagsRepository::new(pg_pool)
}

fn tag_types_repository(pg_pool: PgPool) -> TagTypesRepositoryImpl {
    PostgresTagTypesRepository::new(pg_pool)
}

fn objects_repository(collator: Collator, root_dir: String) -> ObjectsRepositoryImpl {
    FilesystemObjectsRepository::new(Arc::new(collator), root_dir)
}

fn normalizer() -> NormalizerImpl {
    Normalizer::new()
}

fn external_services_service(external_services_repository: ExternalServicesRepositoryImpl) -> ExternalServicesServiceImpl {
    ExternalServicesService::new(external_services_repository)
}

fn media_service(media_repository: MediaRepositoryImpl, objects_repository: ObjectsRepositoryImpl, replicas_repository: ReplicasRepositoryImpl, sources_repository: SourcesRepositoryImpl, medium_image_processor: MediumImageProcessorImpl, task_tracker: TaskTracker) -> MediaServiceImpl {
    MediaService::new(media_repository, objects_repository, replicas_repository, sources_repository, medium_image_processor, task_tracker)
}

fn tags_service(tags_repository: TagsRepositoryImpl, tag_types_repository: TagTypesRepositoryImpl) -> TagsServiceImpl {
    TagsService::new(tags_repository, tag_types_repository)
}

fn objects_service(media_service: MediaServiceImpl, media_url_factory: Arc<dyn MediaURLFactoryInterface>) -> ObjectsServiceImpl {
    ObjectsService::new(media_service, media_url_factory)
}

fn thumbnails_service(media_service: MediaServiceImpl) -> ThumbnailsServiceImpl {
    ThumbnailsService::new(media_service)
}

fn query() -> QueryImpl {
    Query::new()
}

fn mutation() -> MutationImpl {
    Mutation::new()
}

fn subscription() -> SubscriptionImpl {
    Subscription::new()
}

fn schema(query: QueryImpl, mutation: MutationImpl, subscription: SubscriptionImpl) -> SchemaBuilderImpl {
    Schema::build(query, mutation, subscription)
}

fn medium_image_processor() -> MediumImageProcessorImpl {
    InMemoryImageProcessor::new(Size::new(240, 240), ImageFormat::WebP, FilterType::CatmullRom)
}

fn graphql_service(schema: SchemaImpl) -> GraphQLServiceImpl {
    GraphQLService::new(schema, "/graphql", "/graphql/subscriptions")
}

fn file_media_url_factory(root_url: String) -> FileMediaURLFactory {
    FileMediaURLFactory::new(root_url)
}

fn noop_media_url_factory() -> NoopMediaURLFactory {
    NoopMediaURLFactory::new()
}

fn thumbnail_url_factory() -> ThumbnailURLFactory {
    ThumbnailURLFactory::new("/thumbnails")
}

pub struct Application;

impl Application {
    pub async fn start() -> anyhow::Result<()> {
        let config = env::init();
        let collator = Collator::try_new(&DataLocale::from(config.global.locale), CollatorOptions::new())
            .context("error instantiating collator")?;

        match config.command {
            Commands::Serve(serve) => {
                let pg_pool = pg_pool().await?;
                let task_tracker = task_tracker();

                let external_services_repository = external_services_repository(pg_pool.clone());
                let media_repository = media_repository(pg_pool.clone());
                let replicas_repository = replicas_repository(pg_pool.clone());
                let sources_repository = sources_repository(pg_pool.clone());
                let tags_repository = tags_repository(pg_pool.clone());
                let tag_types_repository = tag_types_repository(pg_pool);

                let objects_repository = objects_repository(collator, serve.media_root_dir);
                let medium_image_processor = medium_image_processor();

                let external_services_service = external_services_service(external_services_repository);
                let media_service = media_service(media_repository, objects_repository, replicas_repository, sources_repository, medium_image_processor, task_tracker.clone());
                let tags_service = tags_service(tags_repository, tag_types_repository);

                let normalizer = Arc::new(normalizer());
                let media_url_factory: Arc<dyn MediaURLFactoryInterface> = match serve.media_root_url {
                    Some(media_root_url) => Arc::new(file_media_url_factory(media_root_url)),
                    None => Arc::new(noop_media_url_factory()),
                };

                let objects_service = objects_service(media_service.clone(), media_url_factory.clone());

                let thumbnail_url_factory = Arc::new(thumbnail_url_factory());
                let thumbnails_service = thumbnails_service(media_service.clone());

                let query = query();
                let mutation = mutation();
                let subscription = subscription();
                let schema = schema(query, mutation, subscription)
                    .data(external_services_service)
                    .data(media_service)
                    .data(tags_service)
                    .data(normalizer)
                    .data(media_url_factory)
                    .data(thumbnail_url_factory)
                    .finish();

                let graphql_service = graphql_service(schema);

                let tls = Option::zip(serve.tls_cert, serve.tls_key);
                Engine::new(graphql_service, objects_service, thumbnails_service)
                    .start(serve.port, tls)
                    .await?;

                task_tracker.close();
                task_tracker.wait().await;
            },
            Commands::Schema(SchemaCommand { command: SchemaCommands::Print(..) }) => {
                let query = query();
                let mutation = mutation();
                let subscription = subscription();
                let schema = schema(query, mutation, subscription).finish();
                let graphql_service = graphql_service(schema);
                print!("{}", graphql_service.definitions());
            },
            Commands::Migration(migration) => {
                let pg_pool = pg_pool().await?;

                let migrator = Migrator::new();
                let mut conn = pg_pool.acquire().await?;
                migration.command.run(&mut *conn, migrator.into_boxed_migrator()).await?;
            },
        }

        Ok(())
    }
}
