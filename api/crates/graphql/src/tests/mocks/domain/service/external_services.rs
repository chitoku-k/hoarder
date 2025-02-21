use std::future::Future;

use domain::{
    entity::external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
    error::Result,
    iter::CloneableIterator,
    repository::DeleteResult,
    service::external_services::ExternalServicesServiceInterface,
};

mockall::mock! {
    pub(crate) ExternalServicesServiceInterface {}

    impl ExternalServicesServiceInterface for ExternalServicesServiceInterface {
        fn create_external_service<'a, 'b>(&self, slug: &str, kind: ExternalServiceKind, name: &str, base_url: Option<&'a str>, url_pattern: Option<&'b str>) -> impl Future<Output = Result<ExternalService>> + Send;

        fn get_external_services(&self) -> impl Future<Output = Result<Vec<ExternalService>>> + Send;

        #[mockall::concretize]
        fn get_external_services_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<ExternalService>>> + Send
        where
            T: CloneableIterator<Item = ExternalServiceId> + Send;

        fn get_external_services_by_url(&self, url: &str) -> impl Future<Output = Result<Vec<(ExternalService, ExternalMetadata)>>> + Send;

        fn update_external_service_by_id<'a, 'b, 'c, 'd>(&self, id: ExternalServiceId, slug: Option<&'a str>, name: Option<&'b str>, base_url: Option<Option<&'c str>>, url_pattern: Option<Option<&'d str>>) -> impl Future<Output = Result<ExternalService>> + Send;

        fn delete_external_service_by_id(&self, id: ExternalServiceId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }
}
