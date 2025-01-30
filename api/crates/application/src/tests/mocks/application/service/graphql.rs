use std::future::Future;

use axum::{body::Body, http::Request, response::Response};

use crate::service::graphql::{GraphQLEndpoints, GraphQLServiceInterface};

mockall::mock! {
    pub(crate) GraphQLServiceInterface {}

    impl GraphQLServiceInterface for GraphQLServiceInterface {
        fn execute(&self, req: Request<Body>) -> impl Future<Output = Response> + Send;

        fn subscriptions(&self, req: Request<Body>) -> impl Future<Output = Response> + Send;

        fn endpoints(&self) -> GraphQLEndpoints<'static>;

        fn graphiql(&self) -> Response;

        fn definitions(&self) -> String;
    }
}
