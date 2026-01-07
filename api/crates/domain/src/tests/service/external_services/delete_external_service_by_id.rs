use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::external_services::ExternalServiceId,
    error::{Error, ErrorKind},
    repository::DeleteResult,
    service::external_services::{ExternalServicesService, ExternalServicesServiceInterface},
};

use super::mocks::domain::repository::external_services::MockExternalServicesRepository;

#[tokio::test]
async fn succeeds() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.delete_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
    ).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Box::pin(err(Error::other("error communicating with database"))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.delete_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
