use async_trait::async_trait;
use derive_more::Constructor;

use crate::{
    entity::external_services::{ExternalService, ExternalServiceId},
    repository::{external_services, DeleteResult},
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
#[async_trait]
pub trait ExternalServicesServiceInterface: Send + Sync + 'static {
    /// Creates an external service.
    async fn create_external_service(&self, slug: &str, name: &str) -> anyhow::Result<ExternalService>;

    /// Gets external services.
    async fn get_external_services(&self) -> anyhow::Result<Vec<ExternalService>>;

    /// Gets the external services by their IDs.
    async fn get_external_services_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<ExternalService>>
    where
        T: IntoIterator<Item = ExternalServiceId> + Send + Sync + 'static;

    /// Updates the external service by ID.
    async fn update_external_service_by_id<'a>(&self, id: ExternalServiceId, name: Option<&'a str>) -> anyhow::Result<ExternalService>;

    /// Deletes the external service by ID.
    async fn delete_external_service_by_id(&self, id: ExternalServiceId) -> anyhow::Result<DeleteResult>;
}

#[derive(Clone, Constructor)]
pub struct ExternalServicesService<ExternalServicesRepository> {
    external_services_repository: ExternalServicesRepository,
}

#[async_trait]
impl<ExternalServicesRepository> ExternalServicesServiceInterface for ExternalServicesService<ExternalServicesRepository>
where
    ExternalServicesRepository: external_services::ExternalServicesRepository,
{
    async fn create_external_service(&self, slug: &str, name: &str) -> anyhow::Result<ExternalService> {
        match self.external_services_repository.create(slug, name).await {
            Ok(service) => Ok(service),
            Err(e) => {
                log::error!("failed to create an external service\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_external_services(&self) -> anyhow::Result<Vec<ExternalService>> {
        match self.external_services_repository.fetch_all().await {
            Ok(services) => Ok(services),
            Err(e) => {
                log::error!("failed to get external services\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_external_services_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<ExternalService>>
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

    async fn update_external_service_by_id<'a>(&self, id: ExternalServiceId, name: Option<&'a str>) -> anyhow::Result<ExternalService> {
        match self.external_services_repository.update_by_id(id, name).await {
            Ok(service) => Ok(service),
            Err(e) => {
                log::error!("failed to update the external service\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_external_service_by_id(&self, id: ExternalServiceId) -> anyhow::Result<DeleteResult> {
        match self.external_services_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the external service\nError: {e:?}");
                Err(e)
            },
        }
    }
}
