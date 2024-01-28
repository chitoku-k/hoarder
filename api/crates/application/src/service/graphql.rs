use std::{future::Future, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::Request,
    response::Response,
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait GraphQLServiceInterface: Send + Sync + 'static {
    fn execute(&self, req: Request<Body>) -> impl Future<Output = Response<Body>> + Send;

    fn endpoint(&self) -> &str;

    fn graphiql(&self) -> Response<Body>;

    fn definitions(&self) -> String;
}

pub(crate) async fn execute<GraphQLService>(graphql_service: State<Arc<GraphQLService>>, req: Request<Body>) -> Response<Body>
where
    GraphQLService: GraphQLServiceInterface,
{
    graphql_service.execute(req).await
}

pub(crate) async fn graphiql<GraphQLService>(graphql_service: State<Arc<GraphQLService>>) -> Response<Body>
where
    GraphQLService: GraphQLServiceInterface,
{
    graphql_service.graphiql()
}
