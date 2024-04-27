use std::future::Future;

use crate::error::Result;

use derive_more::Constructor;

use crate::{
    entity::external_services::{ExternalService, ExternalServiceId},
    repository::{external_services, DeleteResult},
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ExternalServicesServiceInterface: Send + Sync + 'static {
    /// Creates an external service.
    fn create_external_service<'a>(&self, slug: &str, kind: &str, name: &str, base_url: Option<&'a str>) -> impl Future<Output = Result<ExternalService>> + Send;

    /// Gets external services.
    fn get_external_services(&self) -> impl Future<Output = Result<Vec<ExternalService>>> + Send;

    /// Gets the external services by their IDs.
    fn get_external_services_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<ExternalService>>> + Send
    where
        T: IntoIterator<Item = ExternalServiceId> + Send + Sync + 'static;

    /// Updates the external service by ID.
    fn update_external_service_by_id<'a>(&self, id: ExternalServiceId, slug: Option<&'a str>, name: Option<&'a str>, base_url: Option<Option<&'a str>>) -> impl Future<Output = Result<ExternalService>> + Send;

    /// Deletes the external service by ID.
    fn delete_external_service_by_id(&self, id: ExternalServiceId) -> impl Future<Output = Result<DeleteResult>> + Send;
}

#[derive(Clone, Constructor)]
pub struct ExternalServicesService<ExternalServicesRepository> {
    external_services_repository: ExternalServicesRepository,
}

impl<ExternalServicesRepository> ExternalServicesServiceInterface for ExternalServicesService<ExternalServicesRepository>
where
    ExternalServicesRepository: external_services::ExternalServicesRepository,
{
    async fn create_external_service<'a>(&self, slug: &str, kind: &str, name: &str, base_url: Option<&'a str>) -> Result<ExternalService> {
        match self.external_services_repository.create(slug, kind, name, base_url).await {
            Ok(service) => Ok(service),
            Err(e) => {
                log::error!("failed to create an external service\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_external_services(&self) -> Result<Vec<ExternalService>> {
        match self.external_services_repository.fetch_all().await {
            Ok(services) => Ok(services),
            Err(e) => {
                log::error!("failed to get external services\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_external_services_by_ids<T>(&self, ids: T) -> Result<Vec<ExternalService>>
    where
        T: IntoIterator<Item = ExternalServiceId> + Send + Sync + 'static,
    {
        match self.external_services_repository.fetch_by_ids(ids).await {
            Ok(services) => Ok(services),
            Err(e) => {
                log::error!("failed to get external services\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn update_external_service_by_id<'a>(&self, id: ExternalServiceId, slug: Option<&'a str>, name: Option<&'a str>, base_url: Option<Option<&'a str>>) -> Result<ExternalService> {
        match self.external_services_repository.update_by_id(id, slug, name, base_url).await {
            Ok(service) => Ok(service),
            Err(e) => {
                log::error!("failed to update the external service\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_external_service_by_id(&self, id: ExternalServiceId) -> Result<DeleteResult> {
        match self.external_services_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the external service\nError: {e:?}");
                Err(e)
            },
        }
    }
}
