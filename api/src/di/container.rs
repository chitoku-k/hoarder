use std::time::Duration;

use anyhow::Context;
use log::LevelFilter;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};

use crate::{
    application::{
        server::Engine,
        thumbnails::{ThumbnailURLFactory, ThumbnailsHandler},
    },
    domain::service::{
        external_services::ExternalServicesService,
        media::MediaService,
        tags::TagsService,
    },
    infrastructure::{
        config::env,
        repository::{
            external_services::PostgresExternalServicesRepository,
            media::PostgresMediaRepository,
            replicas::PostgresReplicasRepository,
            sources::PostgresSourcesRepository,
            tag_types::PostgresTagTypesRepository,
            tags::PostgresTagsRepository,
        },
    },
};

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
            .connect_timeout(Duration::from_secs(5))
            .connect_with(pg_options)
            .await
            .context("error connecting to database")?;

        let engine = Engine::new(
            config.port,
            Option::zip(config.tls_cert, config.tls_key),
            ThumbnailURLFactory::new("/thumbnails/".into()),
            ThumbnailsHandler::new(
                MediaService::new(
                    PostgresMediaRepository::new(pg_pool.clone()),
                    PostgresReplicasRepository::new(pg_pool.clone()),
                    PostgresSourcesRepository::new(pg_pool.clone()),
                ),
            ),
            ExternalServicesService::new(
                PostgresExternalServicesRepository::new(pg_pool.clone()),
            ),
            MediaService::new(
                PostgresMediaRepository::new(pg_pool.clone()),
                PostgresReplicasRepository::new(pg_pool.clone()),
                PostgresSourcesRepository::new(pg_pool.clone()),
            ),
            TagsService::new(
                PostgresTagsRepository::new(pg_pool.clone()),
                PostgresTagTypesRepository::new(pg_pool),
            ),
        );

        engine.start().await
    }
}
