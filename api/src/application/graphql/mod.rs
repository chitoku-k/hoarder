use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptySubscription, Enum, Schema, Upload,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{response::{self, IntoResponse}, Extension};
use futures::io::AsyncReadExt;

use crate::{
    application::graphql::{mutation::Mutation, query::Query},
    domain::{
        repository,
        service::{
            external_services::ExternalServicesServiceInterface,
            media::MediaServiceInterface,
            tags::TagsServiceInterface,
        },
    },
};

pub mod mutation;
pub mod query;

mod external_services;
mod media;
mod replicas;
mod sources;
mod tags;

pub type APISchema<ExternalServicesService, MediaService, TagsService> = Schema<
    Query<ExternalServicesService, MediaService, TagsService>,
    Mutation<ExternalServicesService, MediaService, TagsService>,
    EmptySubscription,
>;

pub async fn handle<ExternalServicesService, MediaService, TagsService>(
    schema: Extension<APISchema<ExternalServicesService, MediaService, TagsService>>,
    req: GraphQLRequest,
) -> GraphQLResponse
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
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

#[derive(Enum, Clone, Copy, Default, Eq, PartialEq)]
pub enum OrderDirection {
    #[default]
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
    pub fn rev(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}
