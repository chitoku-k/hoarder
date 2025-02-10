use std::{borrow::Cow, future::Future};

use crate::{
    entity::external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
    error::{Error, ErrorKind, Result},
    iter::CloneableIterator,
    repository::{external_services, DeleteResult},
};

use derive_more::Constructor;
use regex::Regex;

pub trait ExternalServicesServiceInterface: Send + Sync + 'static {
    /// Creates an external service.
    fn create_external_service(&self, slug: &str, kind: ExternalServiceKind, name: &str, base_url: Option<&str>, url_pattern: Option<&str>) -> impl Future<Output = Result<ExternalService>> + Send;

    /// Gets external services.
    fn get_external_services(&self) -> impl Future<Output = Result<Vec<ExternalService>>> + Send;

    /// Gets the external services by their IDs.
    fn get_external_services_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<ExternalService>>> + Send
    where
        T: CloneableIterator<Item = ExternalServiceId> + Send;

    /// Gets the external services and metadata by URL.
    fn get_external_services_by_url(&self, url: &str) -> impl Future<Output = Result<Vec<(ExternalService, ExternalMetadata)>>> + Send;

    /// Updates the external service by ID.
    fn update_external_service_by_id(&self, id: ExternalServiceId, slug: Option<&str>, name: Option<&str>, base_url: Option<Option<&str>>, url_pattern: Option<Option<&str>>) -> impl Future<Output = Result<ExternalService>> + Send;

    /// Deletes the external service by ID.
    fn delete_external_service_by_id(&self, id: ExternalServiceId) -> impl Future<Output = Result<DeleteResult>> + Send;
}

fn validate_url_pattern(url_pattern: &str) -> Result<()> {
    if let Err(e) = Regex::new(url_pattern) {
        let url_pattern = url_pattern.to_string();
        let description = Some(e.to_string());
        Err(Error::new(ErrorKind::ExternalServiceUrlPatternInvalid { url_pattern, description }, e))
    } else {
        Ok(())
    }
}

#[derive(Clone, Constructor)]
pub struct ExternalServicesService<ExternalServicesRepository> {
    external_services_repository: ExternalServicesRepository,
}

impl<ExternalServicesRepository> ExternalServicesServiceInterface for ExternalServicesService<ExternalServicesRepository>
where
    ExternalServicesRepository: external_services::ExternalServicesRepository,
{
    #[tracing::instrument(skip_all)]
    async fn create_external_service(&self, slug: &str, kind: ExternalServiceKind, name: &str, base_url: Option<&str>, url_pattern: Option<&str>) -> Result<ExternalService> {
        let base_urls = match base_url {
            Some(base_url) => &[base_url],
            None => kind.default_base_urls(),
        };
        let url_pattern = match url_pattern {
            Some(url_pattern) => Some(url_pattern.into()),
            None => kind.default_url_pattern(base_urls).map(Cow::from),
        };

        if let Some(url_pattern) = &url_pattern {
            validate_url_pattern(url_pattern)?;
        }

        match self.external_services_repository.create(slug, kind, name, base_urls.first().cloned(), url_pattern.as_deref()).await {
            Ok(service) => Ok(service),
            Err(e) => {
                tracing::error!("failed to create an external service\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_external_services(&self) -> Result<Vec<ExternalService>> {
        match self.external_services_repository.fetch_all().await {
            Ok(services) => Ok(services),
            Err(e) => {
                tracing::error!("failed to get external services\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_external_services_by_ids<T>(&self, ids: T) -> Result<Vec<ExternalService>>
    where
        T: CloneableIterator<Item = ExternalServiceId> + Send,
    {
        match self.external_services_repository.fetch_by_ids(ids).await {
            Ok(services) => Ok(services),
            Err(e) => {
                tracing::error!("failed to get external services\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_external_services_by_url(&self, url: &str) -> Result<Vec<(ExternalService, ExternalMetadata)>> {
        match self.external_services_repository.fetch_all().await {
            Ok(external_services) => {
                let external_metadata = external_services
                    .into_iter()
                    .filter_map(|external_service| external_service
                        .metadata_by_url(url)
                        .map(|external_metadata| (external_service, external_metadata)))
                    .collect();

                Ok(external_metadata)
            },
            Err(e) => {
                tracing::error!("failed to get external services\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn update_external_service_by_id(&self, id: ExternalServiceId, slug: Option<&str>, name: Option<&str>, base_url: Option<Option<&str>>, url_pattern: Option<Option<&str>>) -> Result<ExternalService> {
        if let Some(Some(url_pattern)) = url_pattern {
            validate_url_pattern(url_pattern)?;
        }

        match self.external_services_repository.update_by_id(id, slug, name, base_url, url_pattern).await {
            Ok(service) => Ok(service),
            Err(e) => {
                tracing::error!("failed to update the external service\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn delete_external_service_by_id(&self, id: ExternalServiceId) -> Result<DeleteResult> {
        match self.external_services_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::error!("failed to delete the external service\nError: {e:?}");
                Err(e)
            },
        }
    }
}
