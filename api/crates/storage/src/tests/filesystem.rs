use std::ffi::OsStr;

use domain::{
    entity::objects::{Entry, EntryKind, EntryUrl},
    error::ErrorKind,
    repository::{DeleteResult, objects::{ObjectOverwriteBehavior, ObjectStatus, ObjectsRepository}},
};
use futures::TryStreamExt;
use icu_collator::CollatorBorrowed;
use icu_locale_core::Locale;
use pretty_assertions::{assert_eq, assert_matches};
use tempfile::tempdir;
use tokio::{fs::{File, create_dir_all, metadata}, io::{self, AsyncReadExt, AsyncWriteExt}};
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
    let (actual_entry, actual_status, mut actual_write) = repository.put(
        EntryUrl::from_path_str("file://", "/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap();

    assert_eq!(actual_entry.name, "77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", "/77777777-7777-7777-7777-777777777777.png")));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_status, ObjectStatus::Created);

    const BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    actual_write.write_all(BUF).await.unwrap();
    actual_write.flush().await.unwrap();

    let readdir = tokio::fs::read_dir(root_dir.path()).await.unwrap();
    let actual: Vec<_> = ReadDirStream::new(readdir).try_collect().await.unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].file_name(), "77777777-7777-7777-7777-777777777777.png");
    assert!(actual[0].file_type().await.unwrap().is_file());

    let actual = tokio::fs::read(root_dir.path().join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    assert_eq!(actual, BUF);
}

#[tokio::test]
async fn put_succeeds_with_new_directory_and_new_file() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let (actual_entry, actual_status, mut actual_write) = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap();

    assert_eq!(actual_entry.name, "77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_status, ObjectStatus::Created);

    const BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    actual_write.write_all(BUF).await.unwrap();
    actual_write.flush().await.unwrap();

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
async fn put_succeeds_with_existing_directory_and_new_file() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let (actual_entry, actual_status, mut actual_write) = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap();

    assert_eq!(actual_entry.name, "77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_status, ObjectStatus::Created);

    const BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    actual_write.write_all(BUF).await.unwrap();
    actual_write.flush().await.unwrap();

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

    const OLD_BUF: &[u8] = &[0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8];

    let mut file = File::create_new(root_dir.path().join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    file.write_all(OLD_BUF).await.unwrap();
    file.flush().await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let (actual_entry, actual_status, mut actual_write) = repository.put(
        EntryUrl::from_path_str("file://", "/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Overwrite,
    ).await.unwrap();

    assert_eq!(actual_entry.name, "77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", "/77777777-7777-7777-7777-777777777777.png")));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_status, ObjectStatus::Existing);

    const NEW_BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    actual_write.set_len(0).await.unwrap();
    actual_write.write_all(NEW_BUF).await.unwrap();
    actual_write.flush().await.unwrap();

    let readdir = tokio::fs::read_dir(root_dir.path()).await.unwrap();
    let actual: Vec<_> = ReadDirStream::new(readdir).try_collect().await.unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].file_name(), "77777777-7777-7777-7777-777777777777.png");
    assert!(actual[0].file_type().await.unwrap().is_file());

    let actual = tokio::fs::read(root_dir.path().join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    assert_eq!(actual, NEW_BUF);
}

#[tokio::test]
async fn put_succeeds_with_existing_directory_and_existing_file() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();

    const OLD_BUF: &[u8] = &[0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8];

    let mut file = File::create_new(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    file.write_all(OLD_BUF).await.unwrap();
    file.flush().await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let (actual_entry, actual_status, mut actual_write) = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Overwrite,
    ).await.unwrap();

    assert_eq!(actual_entry.name, "77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_status, ObjectStatus::Existing);

    const NEW_BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    actual_write.set_len(0).await.unwrap();
    actual_write.write_all(NEW_BUF).await.unwrap();
    actual_write.flush().await.unwrap();

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
async fn put_succeeds_with_long_filename() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let long_directory = format!("{:1>255}", "");
    let long_filename = format!("{:2>255}", ".png");
    let long_path = format!("/{long_directory}/{long_filename}");

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let (actual_entry, actual_status, mut actual_write) = repository.put(
        EntryUrl::from_path_str("file://", &long_path),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap();

    assert_eq!(actual_entry.name, long_filename);
    assert_eq!(actual_entry.name.len(), 255);
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", &long_path)));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_status, ObjectStatus::Created);

    const BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    actual_write.write_all(BUF).await.unwrap();
    actual_write.flush().await.unwrap();

    let readdir = tokio::fs::read_dir(root_dir.path()).await.unwrap();
    let actual: Vec<_> = ReadDirStream::new(readdir).try_collect().await.unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].file_name(), OsStr::new(&long_directory));
    assert!(actual[0].file_type().await.unwrap().is_dir());

    let readdir = tokio::fs::read_dir(root_dir.path().join(&long_directory)).await.unwrap();
    let actual: Vec<_> = ReadDirStream::new(readdir).try_collect().await.unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].file_name(), OsStr::new(&long_filename));
    assert!(actual[0].file_type().await.unwrap().is_file());

    let actual = tokio::fs::read(root_dir.path().join(&long_directory).join(&long_filename)).await.unwrap();
    assert_eq!(actual, BUF);
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
async fn put_fails_with_existing_file_by_creating_directory() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    File::create_new(root_dir.path().join("ゆるゆり")).await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.put(
        EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png"),
        ObjectOverwriteBehavior::Fail,
    ).await.unwrap_err();

    let expected_url = "file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/77777777-7777-7777-7777-777777777777.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectAlreadyExists { url, entry } if url == expected_url && entry.is_none());
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
async fn get_succeeds() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();

    const BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    let mut file = File::create_new(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    file.write_all(BUF).await.unwrap();
    file.flush().await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let (actual_entry, mut actual_file) = repository.get(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    let actual_metadata = actual_entry.metadata.unwrap();

    assert_eq!(actual_entry.name, "77777777-7777-7777-7777-777777777777.png");
    assert_eq!(actual_entry.url, Some(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")));
    assert_eq!(actual_entry.kind, EntryKind::Object);
    assert_eq!(actual_metadata.size, BUF.len() as u64);

    let mut actual_buf = Vec::with_capacity(BUF.len());
    actual_file.read_to_end(&mut actual_buf).await.unwrap();

    assert_eq!(actual_buf, BUF);
}

#[tokio::test]
async fn get_fails_with_parent_directory() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();

    const BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    let mut file = File::create_new(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    file.write_all(BUF).await.unwrap();
    file.flush().await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.get(EntryUrl::from_path_str("file://", "/../ゆるゆり/77777777-7777-7777-7777-777777777777.png")).await.unwrap_err();

    let expected_url = "file:///../%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/77777777-7777-7777-7777-777777777777.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn get_fails_with_not_found() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.get(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")).await.unwrap_err();

    let expected_url = "file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/77777777-7777-7777-7777-777777777777.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectNotFound { url } if url == expected_url);
}

#[tokio::test]
async fn get_fails_with_invalid_filename() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.get(EntryUrl::from_path_str("file://", "/ゆるゆり/\x00.png")).await.unwrap_err();

    let expected_url = "file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A/%00.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn copy_succeeds() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    const OLD_BUF: &[u8] = &[0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8];

    let mut file = File::create_new(root_dir.path().join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    file.write_all(OLD_BUF).await.unwrap();
    file.flush().await.unwrap();

    const NEW_BUF: &[u8] = &[0x00, 0x01, 0x02, 0x03];

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual_written = {
        let mut read = NEW_BUF;
        let mut write = file;
        repository.copy(&mut read, &mut write).await.unwrap()
    };

    assert_eq!(actual_written, NEW_BUF.len() as u64);

    let mut actual_buf = Vec::with_capacity(NEW_BUF.len());
    let mut file = File::open(root_dir.path().join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    file.read_to_end(&mut actual_buf).await.unwrap();

    assert_eq!(actual_buf, NEW_BUF);
}

#[tokio::test]
async fn list_succeeds_with_empty() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.list(EntryUrl::from_path_str("file://", "/")).await.unwrap();

    assert_eq!(actual, Vec::new());
}

#[tokio::test]
async fn list_succeeds_with_entries() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();
    File::create_new(root_dir.path().join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.list(EntryUrl::from_path_str("file://", "/")).await.unwrap();

    assert_eq!(actual, vec![
        Entry::new(
            "77777777-7777-7777-7777-777777777777.png".to_string(),
            Some(EntryUrl::from_path_str("file://", "/77777777-7777-7777-7777-777777777777.png")),
            EntryKind::Object,
            None,
        ),
        Entry::new(
            "ゆるゆり".to_string(),
            Some(EntryUrl::from_path_str("file://", "/ゆるゆり")),
            EntryKind::Container,
            None,
        ),
    ]);
}

#[tokio::test]
async fn list_fails_with_parent_directory() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.list(EntryUrl::from_path_str("file://", "/../")).await.unwrap_err();

    let expected_url = "file:///../";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn list_fails_with_not_found() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.list(EntryUrl::from_path_str("file://", "/ゆるゆり")).await.unwrap_err();

    let expected_url = "file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A";
    assert_matches!(actual.kind(), ErrorKind::ObjectNotFound { url } if url == expected_url);
}

#[tokio::test]
async fn list_fails_with_invalid_filename() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.list(EntryUrl::from_path_str("file://", "/\x00")).await.unwrap_err();

    let expected_url = "file:///%00";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn delete_succeeds_with_file() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();
    File::create_new(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.delete(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual = metadata(root_dir.path().join("ゆるゆり")).await.unwrap();
    assert!(actual.is_dir());

    let actual = metadata(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap_err();
    assert_matches!(actual.kind(), io::ErrorKind::NotFound);
}

#[tokio::test]
async fn delete_succeeds_with_not_found() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.delete(EntryUrl::from_path_str("file://", "/ゆるゆり/77777777-7777-7777-7777-777777777777.png")).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[tokio::test]
async fn delete_fails_with_invalid_filename() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.delete(EntryUrl::from_path_str("file://", "/\x00/77777777-7777-7777-7777-777777777777.png")).await.unwrap_err();

    let expected_url = "file:///%00/77777777-7777-7777-7777-777777777777.png";
    assert_matches!(actual.kind(), ErrorKind::ObjectUrlInvalid { url } if url == expected_url);
}

#[tokio::test]
async fn delete_fails_with_directory() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    create_dir_all(root_dir.path().join("ゆるゆり")).await.unwrap();
    File::create_new(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.delete(EntryUrl::from_path_str("file://", "/ゆるゆり")).await.unwrap_err();

    let expected_url = "file:///%E3%82%86%E3%82%8B%E3%82%86%E3%82%8A";
    assert_matches!(actual.kind(), ErrorKind::ObjectDeleteFailed { url } if url == expected_url);

    let actual = metadata(root_dir.path().join("ゆるゆり")).await.unwrap();
    assert!(actual.is_dir());

    let actual = metadata(root_dir.path().join("ゆるゆり").join("77777777-7777-7777-7777-777777777777.png")).await.unwrap();
    assert!(actual.is_file());
}
