use async_trait::async_trait;
use axum::{
    body::{BoxBody, Body},
    extract::State,
    http::Request,
    response::Response,
};

#[async_trait]
pub trait GraphQLServiceInterface: Send + Sync + 'static {
    async fn execute(&self, req: Request<Body>) -> Response<BoxBody>;

    fn endpoint(&self) -> &str;

    fn graphiql(&self) -> Response<BoxBody>;

    fn definitions(&self) -> String;
}

pub(crate) async fn execute<GraphQLService>(graphql_service: State<GraphQLService>, req: Request<Body>) -> Response<BoxBody>
where
    GraphQLService: GraphQLServiceInterface,
{
    graphql_service.execute(req).await
}

pub(crate) async fn graphiql<GraphQLService>(graphql_service: State<GraphQLService>) -> Response<BoxBody>
where
    GraphQLService: GraphQLServiceInterface,
{
    graphql_service.graphiql()
}
