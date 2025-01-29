#![allow(clippy::too_many_arguments)]

use application::service::graphql::{GraphQLEndpoints, GraphQLServiceInterface};
use async_graphql::{http::GraphiQLSource, Enum, ObjectType, SubscriptionType};
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    body::Body,
    http::Request,
    response::{IntoResponse, Html, Response},
};
use derive_more::Constructor;
use domain::repository;
use tower_service::Service;

pub use async_graphql::{Schema, SchemaBuilder};

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

#[derive(Clone, Constructor)]
pub struct GraphQLService<Query, Mutation, Subscription> {
    schema: Schema<Query, Mutation, Subscription>,
    graphql_endpoint: &'static str,
    subscriptions_endpoint: &'static str,
}

impl<Query, Mutation, Subscription> GraphQLServiceInterface for GraphQLService<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
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

/// The ordering direction.
#[derive(Enum, Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum Order {
    /// Ascending.
    #[default]
    Asc,
    /// Descending.
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

#[cfg(test)]
mod tests {
    use async_graphql::{EmptyMutation, Object, EmptySubscription};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Default)]
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> &str {
            "OK"
        }
    }

    #[test]
    fn graphql_endpoints() {
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let service = GraphQLService::new(schema, "/graphql", "/graphql/subscriptions");

        let actual = service.endpoints();
        assert_eq!(actual.graphql, "/graphql");
        assert_eq!(actual.subscriptions, "/graphql/subscriptions");
    }

    #[test]
    fn graphql_definitions() {
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let service = GraphQLService::new(schema, "/graphql", "/graphql/subscriptions");

        let actual = service.definitions();
        assert!(actual.contains(indoc! {"
            type Query {
            	value: String!
            }
        "}));
    }
}
