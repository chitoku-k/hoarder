use std::{
    fs::{FileType, Metadata},
    path::{Path, PathBuf},
    sync::Arc,
};

use derive_more::Constructor;
use domain::{
    entity::objects::{Entry, EntryMetadata, EntryKind, EntryPath, EntryUrl},
    error::{Error, ErrorKind, Result},
    repository::{objects::{ObjectOverwriteBehavior, ObjectsRepository}, DeleteResult},
};
use futures::{TryFutureExt, TryStreamExt};
use icu::collator::Collator;
use normalize_path::NormalizePath;
use tokio::{
    fs::{canonicalize, read_dir, remove_file, DirBuilder, DirEntry, File},
    io::{self, copy, AsyncRead},
};
use tokio_stream::wrappers::ReadDirStream;

const URL_PREFIX: &str = "file://";

enum FileEntryType<'a> {
    DirEntry(&'a DirEntry),
    File(&'a File),
}

fn entry_kind(file_type: FileType) -> EntryKind {
    if file_type.is_dir() {
        EntryKind::Container
    } else {
        EntryKind::Object
    }
}

async fn entry(path: impl AsRef<Path>, entry: FileEntryType<'_>) -> Result<Entry> {
    let path = path.as_ref();
    let url = format!("{}{}", URL_PREFIX, Path::new("/").join(path).to_string_lossy());

    match entry {
        FileEntryType::DirEntry(entry) => {
            let file_name = entry.file_name();
            let file_type = entry.file_type().await.map_err(Error::other)?;

            Ok(Entry::new(
                file_name.to_string_lossy().into_owned(),
                EntryPath::from(path.join(file_name).to_string_lossy().into_owned()),
                EntryUrl::from(url),
                entry_kind(file_type),
                None,
            ))
        },
        FileEntryType::File(file) => {
            let file_name = path.file_name().ok_or(ErrorKind::ObjectPathInvalid { path: path.to_string_lossy().into_owned() })?;
            let metadata = file.metadata().await.map_err(Error::other)?;
            let file_type = metadata.file_type();
            let entry_metadata = entry_metadata(&metadata)?;

            Ok(Entry::new(
                file_name.to_string_lossy().into_owned(),
                EntryPath::from(path.to_string_lossy().into_owned()),
                EntryUrl::from(url),
                entry_kind(file_type),
                Some(entry_metadata),
            ))
        },
    }
}

fn entry_metadata(metadata: &Metadata) -> Result<EntryMetadata> {
    let len = metadata.len();
    let created = metadata.created().map_err(Error::other)?;
    let modified = metadata.modified().map_err(Error::other)?;
    let accessed = metadata.accessed().map_err(Error::other)?;
    Ok(EntryMetadata::new(len, created.into(), modified.into(), accessed.into()))
}

fn url_to_path(url: &EntryUrl) -> Result<EntryPath> {
    let path = url.strip_prefix(URL_PREFIX).ok_or_else(|| ErrorKind::ObjectUrlUnsupported { url: url.to_string() })?;
    Ok(EntryPath::from(path.to_string()))
}

fn normalize(path: &EntryPath) -> Result<PathBuf> {
    let path = path.strip_prefix('/').ok_or_else(|| ErrorKind::ObjectPathInvalid { path: path.to_string() })?;
    let path = Path::new(path).try_normalize().ok_or_else(|| ErrorKind::ObjectPathInvalid { path: path.to_string() })?;
    Ok(path)
}

#[derive(Clone, Constructor)]
pub struct FilesystemObjectsRepository {
    collator: Arc<Collator>,
    root_dir: String,
}

impl ObjectsRepository for FilesystemObjectsRepository {
    type Read = File;

    async fn put<R>(&self, path: &EntryPath, mut content: R, overwrite: ObjectOverwriteBehavior) -> Result<Entry>
    where
        R: AsyncRead + Send + Unpin,
    {
        let path = normalize(path)?;
        let fullpath = Path::new(&self.root_dir).join(&path);

        if let Some(parent) = fullpath.parent() {
            DirBuilder::new()
                .mode(0o0755)
                .recursive(true)
                .create(parent)
                .await
                .map_err(Error::other)?;
        }

        let result = File::options()
            .create_new(overwrite.is_denied())
            .write(true)
            .open(&fullpath)
            .await;

        let mut file = match result {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                let entry = File::open(&fullpath)
                    .map_err(Error::other)
                    .and_then(|file| {
                        let path = path.clone();
                        async move {
                            let entry = entry(path, FileEntryType::File(&file)).await?;
                            Ok(Box::new(entry))
                        }
                    })
                    .await
                    .ok();

                return Err(ErrorKind::ObjectAlreadyExists { path: path.to_string_lossy().into_owned(), entry })?;
            },
            Err(e) => return Err(Error::new(ErrorKind::ObjectPutFailed { path: path.to_string_lossy().into_owned() }, e))?,
        };

        copy(&mut content, &mut file).await.map_err(Error::other)?;

        let entry = entry(path, FileEntryType::File(&file)).await?;
        Ok(entry)
    }

    async fn get(&self, url: &EntryUrl) -> Result<(Entry, Self::Read)> {
        let path = url_to_path(url)?;
        let path = normalize(&path)?;
        let fullpath = Path::new(&self.root_dir).join(&path);

        match File::open(&fullpath).await {
            Ok(file) => {
                let entry = entry(path, FileEntryType::File(&file)).await?;
                Ok((entry, file))
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(ErrorKind::ObjectNotFound { path: path.to_string_lossy().into_owned() })?,
            Err(e) => Err(Error::new(ErrorKind::ObjectGetFailed { path: path.to_string_lossy().into_owned() }, e))?,
        }
    }

    async fn list(&self, prefix: &EntryPath) -> Result<Vec<Entry>> {
        let prefix = normalize(prefix)?;
        let fullpath = Path::new(&self.root_dir).join(&prefix);

        let readdir = read_dir(&fullpath)
            .await
            .map_err(|e| Error::new(ErrorKind::ObjectListFailed { path: prefix.to_string_lossy().into_owned() }, e))?;

        let canonical_path = canonicalize(&fullpath)
            .await
            .map_err(|e| Error::new(ErrorKind::ObjectGetFailed { path: prefix.to_string_lossy().into_owned() }, e))?
            .strip_prefix(&self.root_dir)
            .map_err(|_| ErrorKind::ObjectPathInvalid { path: prefix.to_string_lossy().into_owned() })?
            .to_owned();

        let mut entries: Vec<_> = ReadDirStream::new(readdir)
            .map_err(|e| Error::new(ErrorKind::ObjectListFailed { path: prefix.to_string_lossy().into_owned() }, e))
            .try_filter_map(|d| {
                let canonical_path = Path::new("/").join(canonical_path.clone());
                async move {
                    let entry = entry(canonical_path, FileEntryType::DirEntry(&d)).await?;
                    Ok(Some(entry))
                }
            })
            .try_collect()
            .await?;

        entries.sort_by(|a, b| self.collator.compare(&a.name, &b.name));
        Ok(entries)
    }

    async fn delete(&self, url: &EntryUrl) -> Result<DeleteResult> {
        let path = url_to_path(url)?;
        let path = normalize(&path)?;
        let fullpath = Path::new(&self.root_dir).join(&path);

        match remove_file(&fullpath).await {
            Ok(()) => Ok(DeleteResult::Deleted(1)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(DeleteResult::NotFound),
            Err(e) => Err(Error::new(ErrorKind::ObjectDeleteFailed { path: path.to_string_lossy().into_owned() }, e))?,
        }
    }
}
