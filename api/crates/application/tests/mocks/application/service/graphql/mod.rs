use std::future::Future;

use application::service::graphql::GraphQLServiceInterface;
use axum::{body::Body, http::Request, response::Response};

mockall::mock! {
    pub GraphQLServiceInterface {}

    impl GraphQLServiceInterface for GraphQLServiceInterface {
        fn execute(&self, req: Request<Body>) -> impl Future<Output = Response> + Send;

        fn endpoint(&self) -> &str;

        fn graphiql(&self) -> Response;

        fn definitions(&self) -> String;
    }
}
