use std::{
    fs::{FileType, Metadata},
    path::{Path, MAIN_SEPARATOR_STR},
};

use domain::{
    entity::objects::{Entry, EntryMetadata, EntryKind, EntryUrl},
    error::Result,
};
use tokio::fs::{DirEntry, File};

use crate::{filesystem::FilesystemEntryUrl, StorageEntry, StorageEntryUrl};

pub(crate) struct FilesystemEntry(Entry);

impl FilesystemEntry {
    pub(crate) async fn from_dir_entry<P>(path: P, entry: &DirEntry) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let (name, url) = Self::path(path.as_ref().join(entry.file_name()));
        let kind = entry.file_type()
            .await
            .map(Self::kind)
            .unwrap_or(EntryKind::Unknown);

        Ok(Self(Entry::new(name, url, kind, None)))
    }

    pub(crate) async fn from_file<P>(path: P, file: &File) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match file.metadata().await.ok() {
            Some(metadata) => Ok(Self::from_metadata(path, &metadata)),
            None => {
                let (name, url) = Self::path(path);
                Ok(Self(Entry::new(name, url, EntryKind::Unknown, None)))
            },
        }
    }

    pub(crate) fn from_metadata<P>(path: P, metadata: &Metadata) -> Self
    where
        P: AsRef<Path>,
    {
        let (name, url) = Self::path(path);
        let kind = Self::kind(metadata.file_type());
        let metadata = Some(Self::metadata(metadata));
        Self(Entry::new(name, url, kind, metadata))
    }

    fn path<P>(path: P) -> (String, Option<EntryUrl>)
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let url = FilesystemEntryUrl::from_path(Path::new(MAIN_SEPARATOR_STR).join(path))
            .map(|url| url.into_url())
            .ok();

        (name, url)
    }

    fn kind(file_type: FileType) -> EntryKind {
        if file_type.is_dir() {
            EntryKind::Container
        } else {
            EntryKind::Object
        }
    }

    fn metadata(metadata: &Metadata) -> EntryMetadata {
        let len = metadata.len();
        let created = metadata.created().map(Into::into).ok();
        let modified = metadata.modified().map(Into::into).ok();
        let accessed = metadata.accessed().map(Into::into).ok();
        EntryMetadata::new(len, created, modified, accessed)
    }
}

impl StorageEntry for FilesystemEntry {
    fn into_entry(self) -> Entry {
        self.0
    }
}
