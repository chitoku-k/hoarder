#![allow(clippy::too_many_arguments)]

use application::service::graphql::GraphQLServiceInterface;
use async_graphql::{http::GraphiQLSource, Enum, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::FromRequest,
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

#[derive(Clone, Constructor)]
pub struct GraphQLService<ExternalServicesService, MediaService, TagsService> {
    schema: APISchema<ExternalServicesService, MediaService, TagsService>,
    endpoint: &'static str,
}

#[async_trait]
impl<ExternalServicesService, MediaService, TagsService> GraphQLServiceInterface for GraphQLService<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
{
    async fn execute(&self, req: Request<Body>) -> Response<Body> {
        let req: GraphQLRequest = match GraphQLRequest::from_request(req, &()).await {
            Ok(req) => req,
            Err(rejection) => return rejection.into_response(),
        };
        let req = req.into_inner();

        let res = self.schema.execute(req).await;
        let res = GraphQLResponse::from(res);
        res.into_response()
    }

    fn endpoint(&self) -> &str {
        self.endpoint
    }

    fn graphiql(&self) -> Response<Body> {
        let res = GraphiQLSource::build().endpoint(self.endpoint).finish();
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
