use std::{
    fs::{FileType, Metadata},
    path::{Path, MAIN_SEPARATOR_STR},
};

use domain::{
    entity::objects::{Entry, EntryMetadata, EntryKind},
    error::{Error, Result},
};
use tokio::fs::{DirEntry, File};

use crate::{filesystem::FilesystemEntryUrl, StorageEntry, StorageEntryUrl};

pub(crate) struct FilesystemEntry(Entry);

impl FilesystemEntry {
    pub(crate) async fn from_dir_entry<P>(path: P, entry: &DirEntry) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let name = entry.file_name().to_string_lossy().into_owned();
        let url = FilesystemEntryUrl::from_path(Path::new(MAIN_SEPARATOR_STR).join(path).join(entry.file_name()))
            .map(|url| url.into_url())
            .ok();
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
        let path = path.as_ref();
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned().to_string();
        let url = FilesystemEntryUrl::from_path(Path::new(MAIN_SEPARATOR_STR).join(path))
            .map(|url| url.into_url())
            .ok();
        let (kind, metadata) = file.metadata()
            .await
            .map(|metadata| {
                let kind = Self::kind(metadata.file_type());
                let metadata = Self::metadata(&metadata).ok();
                (kind, metadata)
            })
            .unwrap_or((EntryKind::Unknown, None));

        Ok(Self(Entry::new(name, url, kind, metadata)))
    }

    fn kind(file_type: FileType) -> EntryKind {
        if file_type.is_dir() {
            EntryKind::Container
        } else {
            EntryKind::Object
        }
    }

    fn metadata(metadata: &Metadata) -> Result<EntryMetadata> {
        let len = metadata.len();
        let created = metadata.created().map_err(Error::other)?;
        let modified = metadata.modified().map_err(Error::other)?;
        let accessed = metadata.accessed().map_err(Error::other)?;
        Ok(EntryMetadata::new(len, created.into(), modified.into(), accessed.into()))
    }
}

impl StorageEntry for FilesystemEntry {
    fn into_entry(self) -> Entry {
        self.0
    }
}
