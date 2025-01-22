#![allow(clippy::too_many_arguments)]

use application::service::graphql::{GraphQLEndpoints, GraphQLServiceInterface};
use async_graphql::{http::GraphiQLSource, Enum, Schema};
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    body::Body,
    http::Request,
    response::{IntoResponse, Html, Response},
};
use derive_more::Constructor;
use domain::{
    repository,
    service::{
        external_services::ExternalServicesServiceInterface,
        media::MediaServiceInterface,
        tags::TagsServiceInterface,
    },
};
use normalizer::NormalizerInterface;
use tower_service::Service;

use crate::{mutation::Mutation, query::Query, subscription::Subscription};

pub mod error;
pub mod mutation;
pub mod query;
pub mod subscription;

pub mod external_services;
pub mod media;
pub mod objects;
pub mod replicas;
pub mod sources;
pub mod tags;

pub type APISchema<ExternalServicesService, MediaService, TagsService, Normalizer> = Schema<
    Query<ExternalServicesService, MediaService, TagsService, Normalizer>,
    Mutation<ExternalServicesService, MediaService, TagsService, Normalizer>,
    Subscription<MediaService>,
>;

#[derive(Clone, Constructor)]
pub struct GraphQLService<ExternalServicesService, MediaService, TagsService, Normalizer> {
    schema: APISchema<ExternalServicesService, MediaService, TagsService, Normalizer>,
    graphql_endpoint: &'static str,
    subscriptions_endpoint: &'static str,
}

impl<ExternalServicesService, MediaService, TagsService, Normalizer> GraphQLServiceInterface for GraphQLService<ExternalServicesService, MediaService, TagsService, Normalizer>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
    Normalizer: NormalizerInterface,
{
    async fn execute(&self, req: Request<Body>) -> Response {
        GraphQL::new(self.schema.clone()).call(req).await.unwrap()
    }

    async fn subscriptions(&self, req: Request<Body>) -> Response {
        GraphQLSubscription::new(self.schema.clone()).call(req).await.unwrap()
    }

    fn endpoints(&self) -> GraphQLEndpoints<'_> {
        GraphQLEndpoints::new(self.graphql_endpoint, self.subscriptions_endpoint)
    }

    fn graphiql(&self) -> Response {
        let res = GraphiQLSource::build()
            .endpoint(self.graphql_endpoint)
            .subscription_endpoint(self.subscriptions_endpoint)
            .finish();
        let res = Html(res);
        res.into_response()
    }

    fn definitions(&self) -> String {
        self.schema.sdl()
    }
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
    pub const fn rev(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}
