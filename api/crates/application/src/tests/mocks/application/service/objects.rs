use axum::response::Response;

use crate::{Precondition, service::objects::ObjectsServiceInterface};

mockall::mock! {
    pub(crate) ObjectsServiceInterface {}

    impl ObjectsServiceInterface for ObjectsServiceInterface {
        fn serve(&self, url: String, precondition: Option<Precondition>) -> impl Future<Output = Response> + Send;
    }
}
