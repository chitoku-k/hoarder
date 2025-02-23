use crate::{
    entity::external_services::{ExternalService, ExternalServiceId, ExternalServiceKind},
    error::Result,
    iter::CloneableIterator,
    repository::{external_services::ExternalServicesRepository, DeleteResult},
};

mockall::mock! {
    pub(crate) ExternalServicesRepository {}

    impl ExternalServicesRepository for ExternalServicesRepository {
        fn create<'a, 'b>(&self, slug: &str, kind: ExternalServiceKind, name: &str, base_url: Option<&'a str>, url_pattern: Option<&'b str>) -> impl Future<Output = Result<ExternalService>> + Send;

        #[mockall::concretize]
        fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<ExternalService>>> + Send
        where
            T: CloneableIterator<Item = ExternalServiceId> + Send;

        fn fetch_all(&self) -> impl Future<Output = Result<Vec<ExternalService>>> + Send;

        fn update_by_id<'a, 'b, 'c, 'd>(&self, id: ExternalServiceId, slug: Option<&'a str>, name: Option<&'b str>, base_url: Option<Option<&'c str>>, url_pattern: Option<Option<&'d str>>) -> impl Future<Output = Result<ExternalService>> + Send;

        fn delete_by_id(&self, id: ExternalServiceId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for ExternalServicesRepository {
        fn clone(&self) -> Self;
    }
}
