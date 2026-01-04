use application::service::media::MediaURLFactoryInterface;
use pretty_assertions::assert_eq;

use crate::{FileMediaURLFactory, NoopMediaURLFactory};

#[test]
fn file_media_url_factory_public_url_succeeds() {
    let factory = FileMediaURLFactory::new("https://original.example.com".to_string());

    let actual = factory.public_url("file:///77777777-7777-7777-7777-777777777777.png").unwrap();
    assert_eq!(actual, "https://original.example.com/77777777-7777-7777-7777-777777777777.png");
}

#[test]
fn noop_media_url_factory_public_url_succeeds() {
    let factory = NoopMediaURLFactory::new();

    let actual = factory.public_url("file:///77777777-7777-7777-7777-777777777777.png");
    assert!(actual.is_none());
}
