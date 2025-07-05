use application::service::graphql::GraphQLServiceInterface;
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use indoc::indoc;
use pretty_assertions::assert_eq;

use crate::GraphQLService;

mod mocks;

mod mutation;
mod query;
mod subscription;

mod media;
mod sources;
mod tags;

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
