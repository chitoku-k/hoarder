use std::{fs::File as StdFile, io::{self, Read}, path::{Path, PathBuf}, sync::Arc};

use derive_more::Constructor;
use domain::{
    entity::objects::{Entry, EntryKind, EntryUrl},
    error::{Error, ErrorKind, Result},
    repository::{objects::{ObjectOverwriteBehavior, ObjectStatus, ObjectsRepository}, DeleteResult},
};
use futures::{TryFutureExt, TryStreamExt};
use icu_collator::CollatorBorrowed;
use tokio::fs::{canonicalize, read_dir, remove_file, DirBuilder, File};
use tokio_stream::wrappers::ReadDirStream;

use crate::{filesystem::{entry::FilesystemEntry, url::FilesystemEntryUrl}, StorageEntry, StorageEntryUrl};

mod entry;
mod url;

#[derive(Clone, Constructor)]
pub struct FilesystemObjectsRepository {
    collator: Arc<CollatorBorrowed<'static>>,
    root_dir: String,
}

impl FilesystemObjectsRepository {
    fn fullpath<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        Path::new(&self.root_dir).join(path.as_ref())
    }

    async fn mkdir<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        DirBuilder::new()
            .mode(0o0755)
            .recursive(true)
            .create(path)
            .await
    }
}

impl ObjectsRepository for FilesystemObjectsRepository {
    type Read = StdFile;
    type Write = StdFile;

    fn scheme() -> &'static str {
        "file"
    }

    #[tracing::instrument(skip_all)]
    async fn put(&self, url: EntryUrl, overwrite: ObjectOverwriteBehavior) -> Result<(Entry, ObjectStatus, Self::Write)> {
        let url = FilesystemEntryUrl::try_from(url)?;
        let fullpath = self.fullpath(url.as_path());

        if Path::new(&self.root_dir) == fullpath {
            return Err(Error::from(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }))?;
        }

        if let Some(parent) = fullpath.parent() {
            match self.mkdir(parent).await {
                Ok(()) => {},
                Err(e) if matches!(e.kind(), io::ErrorKind::InvalidFilename | io::ErrorKind::InvalidInput | io::ErrorKind::NotFound) => {
                    return Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?
                },
                Err(e) => {
                    return Err(Error::new(ErrorKind::ObjectPutFailed { url: url.into_url().into_inner() }, e))?
                },
            }
        }

        let result = File::options()
            .create_new(true)
            .write(true)
            .open(&fullpath)
            .map_ok(|file| (file, ObjectStatus::Created))
            .or_else(|e| async {
                if e.kind() == io::ErrorKind::AlreadyExists && overwrite.is_overwrite() {
                    let file = File::options()
                        .write(true)
                        .open(&fullpath)
                        .await?;

                    Ok((file, ObjectStatus::Existing))
                } else {
                    Err(e)
                }
            })
            .await;

        let (file, status) = match result {
            Ok((file, status)) => (file, status),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                let entry = File::open(&fullpath)
                    .map_err(Error::other)
                    .and_then(|file| {
                        let path = url.as_path();
                        async move {
                            let entry = FilesystemEntry::from_file(path, &file).await?;
                            Ok(Box::new(entry.into_entry()))
                        }
                    })
                    .await
                    .ok()
                    .filter(|entry| entry.kind == EntryKind::Object);

                return Err(Error::new(ErrorKind::ObjectAlreadyExists { url: url.into_url().into_inner(), entry }, e))?;
            },
            Err(e) if e.kind() == io::ErrorKind::IsADirectory => {
                return Err(Error::new(ErrorKind::ObjectAlreadyExists { url: url.into_url().into_inner(), entry: None }, e))?
            },
            Err(e) if matches!(e.kind(), io::ErrorKind::InvalidFilename | io::ErrorKind::InvalidInput | io::ErrorKind::NotFound) => {
                return Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?
            },
            Err(e) => {
                return Err(Error::new(ErrorKind::ObjectPutFailed { url: url.into_url().into_inner() }, e))?
            },
        };

        let entry = FilesystemEntry::from_file(url.as_path(), &file).await?;
        Ok((entry.into_entry(), status, file.into_std().await))
    }

    #[tracing::instrument(skip_all)]
    async fn get(&self, url: EntryUrl) -> Result<(Entry, Self::Read)> {
        let url = FilesystemEntryUrl::try_from(url)?;
        let fullpath = self.fullpath(url.as_path());

        match File::open(&fullpath).await {
            Ok(file) => {
                let entry = FilesystemEntry::from_file(url.as_path(), &file).await?;
                Ok((entry.into_entry(), file.into_std().await))
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(Error::new(ErrorKind::ObjectNotFound { url: url.into_url().into_inner() }, e))?,
            Err(e) if matches!(e.kind(), io::ErrorKind::InvalidFilename | io::ErrorKind::InvalidInput) => Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?,
            Err(e) => Err(Error::new(ErrorKind::ObjectGetFailed { url: url.into_url().into_inner() }, e))?,
        }
    }

    #[tracing::instrument(skip_all)]
    fn copy<R>(&self, read: &mut R, write: &mut Self::Write) -> Result<u64>
    where
        R: Read,
    {
        write.set_len(0).map_err(Error::other)?;

        let written = io::copy(read, write).map_err(Error::other)?;
        Ok(written)
    }

    #[tracing::instrument(skip_all)]
    async fn list(&self, prefix: EntryUrl) -> Result<Vec<Entry>> {
        let url = FilesystemEntryUrl::try_from(prefix)?;
        let fullpath = self.fullpath(url.as_path());

        let readdir = match read_dir(&fullpath).await {
            Ok(readdir) => readdir,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(Error::new(ErrorKind::ObjectNotFound { url: url.into_url().into_inner() }, e))?,
            Err(e) if matches!(e.kind(), io::ErrorKind::InvalidFilename | io::ErrorKind::InvalidInput) => return Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?,
            Err(e) => return Err(Error::new(ErrorKind::ObjectListFailed { url: url.into_url().into_inner() }, e))?,
        };

        let canonical_path = Path::new("/").join(canonicalize(&fullpath)
            .await
            .map_err(|e| Error::new(ErrorKind::ObjectGetFailed { url: url.to_string() }, e))?
            .strip_prefix(&self.root_dir)
            .map_err(|e| Error::new(ErrorKind::ObjectUrlInvalid { url: url.to_string() }, e))?);

        let mut entries: Vec<_> = ReadDirStream::new(readdir)
            .map_err(|e| Error::new(ErrorKind::ObjectListFailed { url: url.to_string() }, e))
            .try_filter_map(|dir| {
                let canonical_path = canonical_path.as_path();
                async move {
                    let entry = FilesystemEntry::from_dir_entry(canonical_path, &dir).await?;
                    Ok(Some(entry.into_entry()))
                }
            })
            .try_collect()
            .await?;

        entries.sort_by(|a, b| self.collator.compare(&a.name, &b.name));
        Ok(entries)
    }

    #[tracing::instrument(skip_all)]
    async fn delete(&self, url: EntryUrl) -> Result<DeleteResult> {
        let url = FilesystemEntryUrl::try_from(url)?;
        let fullpath = self.fullpath(url.as_path());

        match remove_file(&fullpath).await {
            Ok(()) => Ok(DeleteResult::Deleted(1)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(DeleteResult::NotFound),
            Err(e) if matches!(e.kind(), io::ErrorKind::InvalidFilename | io::ErrorKind::InvalidInput) => Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?,
            Err(e) => Err(Error::new(ErrorKind::ObjectDeleteFailed { url: url.into_url().into_inner() }, e))?,
        }
    }
}
