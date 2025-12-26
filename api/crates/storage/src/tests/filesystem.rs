use domain::{
    entity::objects::{EntryKind, EntryUrl},
    error::ErrorKind,
    repository::objects::{ObjectOverwriteBehavior, ObjectStatus, ObjectsRepository},
};
use futures::TryStreamExt;
use icu_collator::CollatorBorrowed;
use icu_locale_core::Locale;
use pretty_assertions::{assert_eq, assert_matches};
use tempfile::tempdir;
use tokio::{fs::{File, create_dir_all}, io::AsyncWriteExt};
use tokio_stream::wrappers::ReadDirStream;

use crate::filesystem::FilesystemObjectsRepository;

#[test]
fn scheme_succeeds() {
    let actual = FilesystemObjectsRepository::scheme();
    assert_eq!(actual, "file");
}

#[tokio::test]
async fn put_succeeds_with_new_file() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let (actual_entry, actual_status, actual_write) = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap();

    assert_eq!(actual_entry.name, "77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_status, ObjectStatus::Created);

    const BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    let mut file = File::from_std(actual_write);
    file.write_all(BUF).await.unwrap();
    file.flush().await.unwrap();

    let readdir = tokio::fs::read_dir(root_dir.path()).await.unwrap();
    let actual: Vec<_> = ReadDirStream::new(readdir).try_collect().await.unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].file_name(), "ゆるゆり");
    assert!(actual[0].file_type().await.unwrap().is_dir());

    let readdir = tokio::fs::read_dir(root_dir.path().join("ゆるゆり")).await.unwrap();
    let actual: Vec<_> = ReadDirStream::new(readdir).try_collect().await.unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].file_name(), "77777777-7777-7777-7777-777777777777.png");
    assert!(actual[0].file_type().await.unwrap().is_file());

    let actual = tokio::fs::read(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    assert_eq!(actual, BUF);
}

#[tokio::test]
async fn put_succeeds_with_existing_file() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();

    const OLD_BUF: &[u8] = &[0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8];

    let mut file = File::create_new(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    file.write_all(OLD_BUF).await.unwrap();
    file.flush().await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let (actual_entry, actual_status, actual_write) = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Overwrite,
    ).await.unwrap();

    assert_eq!(actual_entry.name, "77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_status, ObjectStatus::Existing);

    const NEW_BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    let mut file = File::from_std(actual_write);
    file.set_len(0).await.unwrap();
    file.write_all(NEW_BUF).await.unwrap();
    file.flush().await.unwrap();

    let readdir = tokio::fs::read_dir(root_dir.path()).await.unwrap();
    let actual: Vec<_> = ReadDirStream::new(readdir).try_collect().await.unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].file_name(), "ゆるゆり");
    assert!(actual[0].file_type().await.unwrap().is_dir());

    let readdir = tokio::fs::read_dir(root_dir.path().join("ゆるゆり")).await.unwrap();
    let actual: Vec<_> = ReadDirStream::new(readdir).try_collect().await.unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].file_name(), "77777777-7777-7777-7777-777777777777.png");
    assert!(actual[0].file_type().await.unwrap().is_file());

    let actual = tokio::fs::read(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    assert_eq!(actual, NEW_BUF);
}

#[tokio::test]
async fn put_fails_with_parent_directory() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.put(
        EntryUrl::from_path_str("file://", "/../77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap_err();

    let expected_url = "file:///../77777777-7777-7777-7777-777777777777.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn put_fails_with_root_directory() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.put(
        EntryUrl::from_path_str("file://", "/"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap_err();

    let expected_url = "file:///";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn put_fails_with_invalid_directory() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.put(
        EntryUrl::from_path_str("file://", "/\x00/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap_err();

    let expected_url = "file:///%00/77777777-7777-7777-7777-777777777777.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn put_fails_with_existing_directory_by_creating_file() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap_err();

    let expected_url = "file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/77777777-7777-7777-7777-777777777777.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectAlreadyExists { url, entry } if url == expected_url && entry.is_none());
}

#[tokio::test]
async fn put_fails_with_existing_file_by_creating_file() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();
    File::create_new(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap_err();

    let expected_url = "file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/77777777-7777-7777-7777-777777777777.png";
    let expected_entry_name = "77777777-7777-7777-7777-777777777777.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectAlreadyExists { url, entry } if url == expected_url && entry.as_deref().is_some_and(|entry| entry.name == expected_entry_name));
}

#[tokio::test]
async fn put_fails_with_invalid_filename() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/\x00.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap_err();

    let expected_url = "file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/%00.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn list_succeeds() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.list(EntryUrl::from_path_str("file://", "/")).await.unwrap();

    assert_eq!(actual, Vec::new());
}
