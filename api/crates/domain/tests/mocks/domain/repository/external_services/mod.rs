use std::future::Future;

use domain::{
    entity::external_services::{ExternalService, ExternalServiceId},
    error::Result,
    repository::{external_services::ExternalServicesRepository, DeleteResult},
};

mockall::mock! {
    pub ExternalServicesRepository {}

    impl ExternalServicesRepository for ExternalServicesRepository {
        fn create<'a>(&self, slug: &str, kind: &str, name: &str, base_url: Option<&'a str>, url_pattern: Option<&'a str>) -> impl Future<Output = Result<ExternalService>> + Send;

        fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<ExternalService>>> + Send
        where
            T: IntoIterator<Item = ExternalServiceId> + Send + 'static;

        fn fetch_all(&self) -> impl Future<Output = Result<Vec<ExternalService>>> + Send;

        fn update_by_id<'a>(&self, id: ExternalServiceId, slug: Option<&'a str>, name: Option<&'a str>, base_url: Option<Option<&'a str>>, url_pattern: Option<Option<&'a str>>) -> impl Future<Output = Result<ExternalService>> + Send;

        fn delete_by_id(&self, id: ExternalServiceId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }
}
