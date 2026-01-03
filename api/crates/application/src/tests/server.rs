use std::net::{Ipv6Addr, SocketAddr};

use hyper::{StatusCode, Uri};
use pretty_assertions::{assert_eq, assert_ne};
use reqwest::ClientBuilder;

use crate::{server::Engine, service::graphql::GraphQLEndpoints};

#[cfg(feature = "tls")]
use reqwest::Certificate;
#[cfg(feature = "tls")]
use super::generate_certificate;

use super::{mocks::{self, application::service::{
    graphql::MockGraphQLServiceInterface,
    objects::MockObjectsServiceInterface,
    thumbnails::MockThumbnailsServiceInterface,
}}};

mod graphql;
mod objects;
mod thumbnails;

#[tokio::test]
async fn start_http_succeeds_with_ipv6() {
    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mock_objects_service = MockObjectsServiceInterface::new();
    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let server = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).start(0, None).unwrap();
    let actual = server.handle.listening().await.unwrap();
    assert_eq!(actual.ip(), Ipv6Addr::UNSPECIFIED);
    assert_ne!(actual.port(), 0);

    let uri = Uri::builder()
        .scheme("http")
        .authority(SocketAddr::from((Ipv6Addr::LOCALHOST, actual.port())).to_string())
        .path_and_query("/healthz")
        .build()
        .unwrap();

    let client = ClientBuilder::new().build().unwrap();
    let actual = client.get(uri.to_string()).send().await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    let actual = actual.bytes().await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, "OK");

    server.handle.shutdown();
    server.shutdown.await.unwrap().unwrap();
}

#[cfg(feature = "tls")]
#[cfg_attr(not(unix), ignore = "native-tls does not yet support TLS 1.3 (sfackler/rust-native-tls#140)")]
#[tokio::test]
async fn start_https_succeeds_with_ipv6() {
    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mock_objects_service = MockObjectsServiceInterface::new();
    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let (ca, cert, key) = generate_certificate().await;
    let cert_path = cert.path().to_str().unwrap();
    let key_path = key.path().to_str().unwrap();

    let server = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).start(0, Some((cert_path.to_string(), key_path.to_string()))).unwrap();
    let actual = server.handle.listening().await.unwrap();
    assert_eq!(actual.ip(), Ipv6Addr::UNSPECIFIED);
    assert_ne!(actual.port(), 0);

    let uri = Uri::builder()
        .scheme("https")
        .authority(SocketAddr::from((Ipv6Addr::LOCALHOST, actual.port())).to_string())
        .path_and_query("/healthz")
        .build()
        .unwrap();

    let client = ClientBuilder::new()
        .tls_certs_only(Certificate::from_pem_bundle(&ca).unwrap())
        .build()
        .unwrap();

    let actual = client.get(uri.to_string()).send().await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    let actual = actual.bytes().await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, "OK");

    server.handle.shutdown();
    server.shutdown.await.unwrap().unwrap();
}
