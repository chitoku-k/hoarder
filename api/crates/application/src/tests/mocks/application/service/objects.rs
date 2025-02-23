use axum::response::Response;

use crate::service::objects::ObjectsServiceInterface;

mockall::mock! {
    pub(crate) ObjectsServiceInterface {}

    impl ObjectsServiceInterface for ObjectsServiceInterface {
        fn redirect(&self, url: String) -> impl Future<Output = Response> + Send;
    }
}
