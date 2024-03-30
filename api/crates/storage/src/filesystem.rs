use std::{path::{Path, PathBuf}, sync::Arc};

use derive_more::Constructor;
use domain::{
    entity::objects::{Entry, EntryKind, EntryUrl},
    error::{Error, ErrorKind, Result},
    repository::{objects::{ObjectOverwriteBehavior, ObjectsRepository}, DeleteResult},
};
use futures::{TryFutureExt, TryStreamExt};
use icu_collator::Collator;
use tokio::{
    fs::{canonicalize, read_dir, remove_file, DirBuilder, File},
    io::{self, copy, AsyncRead},
};
use tokio_stream::wrappers::ReadDirStream;

use crate::{filesystem::{entry::FilesystemEntry, url::FilesystemEntryUrl}, StorageEntry, StorageEntryUrl};

mod entry;
mod url;

#[derive(Clone, Constructor)]
pub struct FilesystemObjectsRepository {
    collator: Arc<Collator>,
    root_dir: String,
}

impl FilesystemObjectsRepository {
    fn fullpath<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        Path::new(&self.root_dir).join(path.as_ref())
    }
}

impl ObjectsRepository for FilesystemObjectsRepository {
    type Read = File;

    fn scheme() -> &'static str {
        "file"
    }

    async fn put<R>(&self, url: EntryUrl, mut content: R, overwrite: ObjectOverwriteBehavior) -> Result<Entry>
    where
        R: AsyncRead + Send + Unpin,
    {
        let url = FilesystemEntryUrl::try_from(url)?;
        let fullpath = self.fullpath(url.as_path());

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
            #[cfg(unix)]
            Err(e) if e.raw_os_error().is_some_and(|errno| errno == libc::EISDIR) => {
                return Err(Error::new(ErrorKind::ObjectAlreadyExists { url: url.into_url().into_inner(), entry: None }, e))?
            },
            Err(e) if e.kind() == io::ErrorKind::InvalidInput => {
                return Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?
            },
            Err(e) => {
                return Err(Error::new(ErrorKind::ObjectPutFailed { url: url.into_url().into_inner() }, e))?
            },
        };

        copy(&mut content, &mut file).await.map_err(Error::other)?;

        let entry = FilesystemEntry::from_file(url.as_path(), &file).await?;
        Ok(entry.into_entry())
    }

    async fn get(&self, url: EntryUrl) -> Result<(Entry, Self::Read)> {
        let url = FilesystemEntryUrl::try_from(url)?;
        let fullpath = self.fullpath(url.as_path());

        match File::open(&fullpath).await {
            Ok(file) => {
                let entry = FilesystemEntry::from_file(url.as_path(), &file).await?;
                Ok((entry.into_entry(), file))
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(Error::new(ErrorKind::ObjectNotFound { url: url.into_url().into_inner() }, e))?,
            Err(e) if e.kind() == io::ErrorKind::InvalidInput => Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?,
            Err(e) => Err(Error::new(ErrorKind::ObjectGetFailed { url: url.into_url().into_inner() }, e))?,
        }
    }

    async fn list(&self, prefix: EntryUrl) -> Result<Vec<Entry>> {
        let url = FilesystemEntryUrl::try_from(prefix)?;
        let fullpath = self.fullpath(url.as_path());

        let readdir = match read_dir(&fullpath).await {
            Ok(readdir) => readdir,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(Error::new(ErrorKind::ObjectNotFound { url: url.into_url().into_inner() }, e))?,
            Err(e) if e.kind() == io::ErrorKind::InvalidInput => return Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?,
            Err(e) => return Err(Error::new(ErrorKind::ObjectListFailed { url: url.into_url().into_inner() }, e))?,
        };

        let canonical_path = canonicalize(&fullpath)
            .await
            .map_err(|e| Error::new(ErrorKind::ObjectGetFailed { url: url.to_string() }, e))?
            .strip_prefix(&self.root_dir)
            .map_err(|e| Error::new(ErrorKind::ObjectUrlInvalid { url: url.to_string() }, e))?
            .to_owned();

        let mut entries: Vec<_> = ReadDirStream::new(readdir)
            .map_err(|e| Error::new(ErrorKind::ObjectListFailed { url: url.to_string() }, e))
            .try_filter_map(|dir| {
                let canonical_path = Path::new("/").join(&canonical_path);
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

    async fn delete(&self, url: EntryUrl) -> Result<DeleteResult> {
        let url = FilesystemEntryUrl::try_from(url)?;
        let fullpath = self.fullpath(url.as_path());

        match remove_file(&fullpath).await {
            Ok(()) => Ok(DeleteResult::Deleted(1)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(DeleteResult::NotFound),
            Err(e) if e.kind() == io::ErrorKind::InvalidInput => Err(Error::new(ErrorKind::ObjectUrlInvalid { url: url.into_url().into_inner() }, e))?,
            Err(e) => Err(Error::new(ErrorKind::ObjectDeleteFailed { url: url.into_url().into_inner() }, e))?,
        }
    }
}
