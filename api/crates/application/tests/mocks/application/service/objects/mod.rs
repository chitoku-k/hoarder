use std::future::Future;

use application::service::objects::ObjectsServiceInterface;
use axum::response::Response;

mockall::mock! {
    pub ObjectsServiceInterface {}

    impl ObjectsServiceInterface for ObjectsServiceInterface {
        fn redirect(&self, url: String) -> impl Future<Output = Response> + Send;
    }
}
