use domain::{
    entity::objects::EntryUrl,
    repository::objects::ObjectsRepository,
};
use icu_collator::CollatorBorrowed;
use icu_locale_core::Locale;
use pretty_assertions::assert_eq;
use tempfile::tempdir;

use crate::filesystem::FilesystemObjectsRepository;

#[tokio::test]
async fn list_succeeds() {
    let collator = CollatorBorrowed::try_new(Locale::UNKNOWN.into(), Default::default()).unwrap();
    let root_dir = tempdir().unwrap();

    let repository = FilesystemObjectsRepository::new(collator, root_dir.path()).await.unwrap();
    let actual = repository.list(EntryUrl::from_path_str("file://", "/")).await.unwrap();

    assert_eq!(actual, Vec::new());
}
