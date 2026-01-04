use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use axum::body::Body;
use hyper::{StatusCode, Uri};
use hyper_util::{client::legacy::{Client, connect::HttpConnector}, rt::TokioExecutor};
use pretty_assertions::{assert_eq, assert_ne};

use crate::{server::Engine, service::graphql::GraphQLEndpoints};

#[cfg(feature = "tls")]
use std::time::Duration;
#[cfg(feature = "tls")]
use hyper_openssl::client::legacy::HttpsConnector;
#[cfg(feature = "tls")]
use openssl::{ssl::{SslConnector, SslMethod}, x509::{X509, store::X509StoreBuilder}};
#[cfg(feature = "tls")]
use tokio::fs::rename;
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
async fn start_http_succeeds_with_ipv4() {
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
        .authority(SocketAddr::from((Ipv4Addr::LOCALHOST, actual.port())).to_string())
        .path_and_query("/healthz")
        .build()
        .unwrap();

    let http = HttpConnector::new();
    let client = Client::builder(TokioExecutor::new()).build::<_, Body>(http);

    let actual = client.get(uri).await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    server.handle.shutdown();
    server.shutdown.await.unwrap().unwrap();
}

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

    let http = HttpConnector::new();
    let client = Client::builder(TokioExecutor::new()).build::<_, Body>(http);

    let actual = client.get(uri).await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    server.handle.shutdown();
    server.shutdown.await.unwrap().unwrap();
}

#[cfg(feature = "tls")]
#[tokio::test]
async fn start_https_succeeds_with_ipv4() {
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
        .authority(SocketAddr::from((Ipv4Addr::LOCALHOST, actual.port())).to_string())
        .path_and_query("/healthz")
        .build()
        .unwrap();

    let mut http = HttpConnector::new();
    http.enforce_http(false);

    let mut x509_store_builder = X509StoreBuilder::new().unwrap();
    x509_store_builder.add_cert(X509::from_pem(&ca).unwrap()).unwrap();

    let mut ssl = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl.set_verify_cert_store(x509_store_builder.build()).unwrap();

    let mut https = HttpsConnector::with_connector(http, ssl).unwrap();
    https.set_callback(|config, _uri| {
        config.set_use_server_name_indication(false);
        config.set_verify_hostname(false);
        config.param_mut().set_host("localhost")?;
        Ok(())
    });
    let client = Client::builder(TokioExecutor::new()).build::<_, Body>(https);

    let actual = client.get(uri).await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    server.handle.shutdown();
    server.shutdown.await.unwrap().unwrap();
}

#[cfg(feature = "tls")]
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

    let mut http = HttpConnector::new();
    http.enforce_http(false);

    let mut x509_store_builder = X509StoreBuilder::new().unwrap();
    x509_store_builder.add_cert(X509::from_pem(&ca).unwrap()).unwrap();

    let mut ssl = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl.set_verify_cert_store(x509_store_builder.build()).unwrap();

    let mut https = HttpsConnector::with_connector(http, ssl).unwrap();
    https.set_callback(|config, _uri| {
        config.set_use_server_name_indication(false);
        config.set_verify_hostname(false);
        config.param_mut().set_host("localhost")?;
        Ok(())
    });
    let client = Client::builder(TokioExecutor::new()).build::<_, Body>(https);

    let actual = client.get(uri).await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    server.handle.shutdown();
    server.shutdown.await.unwrap().unwrap();
}

#[cfg(feature = "tls")]
#[tokio::test]
async fn auto_reload_certificate_succeeds() {
    async fn retry<F, Fut, T, E>(mut f: F, times: usize, duration: Duration) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut actual = None;
        for _ in 0..times {
            actual = Some(f().await);
            if actual.as_ref().unwrap().is_ok() {
                break;
            }
            tokio::time::sleep(duration).await;
        }
        actual.unwrap()
    }

    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mock_objects_service = MockObjectsServiceInterface::new();
    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let (ca1, cert1, key1) = generate_certificate().await;
    let cert_path1 = cert1.path().to_str().unwrap();
    let key_path1 = key1.path().to_str().unwrap();

    let server = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).start(0, Some((cert_path1.to_string(), key_path1.to_string()))).unwrap();
    let actual = server.handle.listening().await.unwrap();
    assert_eq!(actual.ip(), Ipv6Addr::UNSPECIFIED);
    assert_ne!(actual.port(), 0);

    let uri = Uri::builder()
        .scheme("https")
        .authority(SocketAddr::from((Ipv6Addr::LOCALHOST, actual.port())).to_string())
        .path_and_query("/healthz")
        .build()
        .unwrap();

    let mut http = HttpConnector::new();
    http.enforce_http(false);

    let mut x509_store_builder = X509StoreBuilder::new().unwrap();
    x509_store_builder.add_cert(X509::from_pem(&ca1).unwrap()).unwrap();

    let mut ssl = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl.set_verify_cert_store(x509_store_builder.build()).unwrap();

    let mut https = HttpsConnector::with_connector(http.clone(), ssl).unwrap();
    https.set_callback(|config, _uri| {
        config.set_use_server_name_indication(false);
        config.set_verify_hostname(false);
        config.param_mut().set_host("localhost")?;
        Ok(())
    });
    let client = Client::builder(TokioExecutor::new()).build::<_, Body>(https);

    let actual = client.get(uri.clone()).await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    let (ca2, cert2, key2) = generate_certificate().await;
    let cert_path2 = cert2.path().to_str().unwrap();
    let key_path2 = key2.path().to_str().unwrap();

    rename(cert_path2, cert_path1).await.unwrap();
    rename(key_path2, key_path1).await.unwrap();

    let mut x509_store_builder = X509StoreBuilder::new().unwrap();
    x509_store_builder.add_cert(X509::from_pem(&ca2).unwrap()).unwrap();

    let mut ssl = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl.set_verify_cert_store(x509_store_builder.build()).unwrap();

    let mut https = HttpsConnector::with_connector(http.clone(), ssl).unwrap();
    https.set_callback(|config, _uri| {
        config.set_use_server_name_indication(false);
        config.set_verify_hostname(false);
        config.param_mut().set_host("localhost")?;
        Ok(())
    });
    let client = Client::builder(TokioExecutor::new()).build::<_, Body>(https);

    let actual = retry(|| client.get(uri.clone()), 10, Duration::from_millis(100)).await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    let (ca3, cert3, key3) = generate_certificate().await;
    let cert_path3 = cert3.path().to_str().unwrap();
    let key_path3 = key3.path().to_str().unwrap();

    rename(cert_path3, cert_path1).await.unwrap();
    rename(key_path3, key_path1).await.unwrap();

    let mut x509_store_builder = X509StoreBuilder::new().unwrap();
    x509_store_builder.add_cert(X509::from_pem(&ca3).unwrap()).unwrap();

    let mut ssl = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl.set_verify_cert_store(x509_store_builder.build()).unwrap();

    let mut https = HttpsConnector::with_connector(http, ssl).unwrap();
    https.set_callback(|config, _uri| {
        config.set_use_server_name_indication(false);
        config.set_verify_hostname(false);
        config.param_mut().set_host("localhost")?;
        Ok(())
    });
    let client = Client::builder(TokioExecutor::new()).build::<_, Body>(https);

    let actual = retry(|| client.get(uri.clone()), 10, Duration::from_millis(100)).await.unwrap();
    assert_eq!(actual.status(), StatusCode::OK);

    server.handle.shutdown();
    server.shutdown.await.unwrap().unwrap();
}
