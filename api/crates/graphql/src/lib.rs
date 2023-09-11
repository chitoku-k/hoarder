#![allow(clippy::too_many_arguments)]

use async_graphql::{
    http::GraphiQLSource,
    Enum, Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{response::{self, IntoResponse}, Extension};
use domain::{
    repository,
    service::{
        external_services::ExternalServicesServiceInterface,
        media::MediaServiceInterface,
        tags::TagsServiceInterface,
    },
};

use crate::{mutation::Mutation, query::Query, subscription::Subscription};

pub mod mutation;
pub mod query;
pub mod subscription;

pub mod external_services;
pub mod media;
pub mod replicas;
pub mod sources;
pub mod tags;

pub type APISchema<ExternalServicesService, MediaService, TagsService> = Schema<
    Query<ExternalServicesService, MediaService, TagsService>,
    Mutation<ExternalServicesService, MediaService, TagsService>,
    Subscription,
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

pub async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[derive(Enum, Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum Order {
    #[default]
    Asc,
    Desc,
}

impl From<repository::Order> for Order {
    fn from(direction: repository::Order) -> Self {
        use repository::Order::*;
        match direction {
            Ascending => Order::Asc,
            Descending => Order::Desc,
        }
    }
}

impl From<Order> for repository::Order {
    fn from(direction: Order) -> Self {
        use Order::*;
        match direction {
            Asc => repository::Order::Ascending,
            Desc => repository::Order::Descending,
        }
    }
}

impl Order {
    pub fn rev(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}
