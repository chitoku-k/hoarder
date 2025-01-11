use std::future::Future;

use domain::{
    entity::external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
    error::Result,
    repository::DeleteResult,
    service::external_services::ExternalServicesServiceInterface,
};

mockall::mock! {
    pub ExternalServicesServiceInterface {}

    impl ExternalServicesServiceInterface for ExternalServicesServiceInterface {
        fn create_external_service<'a>(&self, slug: &str, kind: &str, name: &str, base_url: Option<&'a str>, url_pattern: Option<&'a str>) -> impl Future<Output = Result<ExternalService>> + Send;

        fn get_external_services(&self) -> impl Future<Output = Result<Vec<ExternalService>>> + Send;

        fn get_external_services_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<ExternalService>>> + Send
        where
            T: IntoIterator<Item = ExternalServiceId> + Send + Sync + 'static;

        fn get_external_services_by_url(&self, url: &str) -> impl Future<Output = Result<Vec<(ExternalService, ExternalMetadata)>>> + Send;

        fn update_external_service_by_id<'a>(&self, id: ExternalServiceId, slug: Option<&'a str>, name: Option<&'a str>, base_url: Option<Option<&'a str>>, url_pattern: Option<Option<&'a str>>) -> impl Future<Output = Result<ExternalService>> + Send;

        fn delete_external_service_by_id(&self, id: ExternalServiceId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }
}
