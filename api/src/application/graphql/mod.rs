use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptySubscription, Enum, Schema, Upload,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{self, IntoResponse},
    Extension,
};
use futures::io::AsyncReadExt;

use crate::{
    application::graphql::{mutation::Mutation, query::Query},
    domain::repository,
};

pub mod mutation;
pub mod query;

mod external_services;
mod media;
mod replicas;
mod sources;
mod tags;

pub type APISchema<ExternalServicesRepository, MediaRepository, ReplicasRepository, SourcesRepository, TagsRepository, TagTypesRepository> = Schema<
    Query<ExternalServicesRepository, MediaRepository, ReplicasRepository, SourcesRepository, TagsRepository, TagTypesRepository>,
    Mutation<ExternalServicesRepository, MediaRepository, ReplicasRepository, SourcesRepository, TagsRepository, TagTypesRepository>,
    EmptySubscription,
>;

pub struct GraphQLHandler;

pub async fn handle<ExternalServicesRepository, MediaRepository, ReplicasRepository, SourcesRepository, TagsRepository, TagTypesRepository>(
    schema: Extension<APISchema<ExternalServicesRepository, MediaRepository, ReplicasRepository, SourcesRepository, TagsRepository, TagTypesRepository>>,
    req: GraphQLRequest,
) -> GraphQLResponse
where
    ExternalServicesRepository: repository::external_services::ExternalServicesRepository,
    MediaRepository: repository::media::MediaRepository,
    ReplicasRepository: repository::replicas::ReplicasRepository,
    SourcesRepository: repository::sources::SourcesRepository,
    TagsRepository: repository::tags::TagsRepository,
    TagTypesRepository: repository::tag_types::TagTypesRepository,
{
    schema.execute(req.into_inner()).await.into()
}

pub async fn process_upload(ctx: &Context<'_>, upload: Upload) -> anyhow::Result<Vec<u8>> {
    let value = upload.value(ctx)?;

    let mut buf = Vec::with_capacity(value.size().unwrap_or_default() as usize);
    value.into_async_read().read_to_end(&mut buf).await?;

    Ok(buf)
}

pub async fn playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[derive(Enum, Clone, Copy, Eq, PartialEq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl From<repository::OrderDirection> for OrderDirection {
    fn from(direction: repository::OrderDirection) -> Self {
        use repository::OrderDirection::*;
        match direction {
            Ascending => OrderDirection::Asc,
            Descending => OrderDirection::Desc,
        }
    }
}

impl From<OrderDirection> for repository::OrderDirection {
    fn from(direction: OrderDirection) -> Self {
        use OrderDirection::*;
        match direction {
            Asc => repository::OrderDirection::Ascending,
            Desc => repository::OrderDirection::Descending,
        }
    }
}

impl OrderDirection {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}
