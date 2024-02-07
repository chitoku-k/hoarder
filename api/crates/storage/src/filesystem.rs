use std::{
    fs::FileType,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use derive_more::Constructor;
use domain::{
    entity::objects::{Entry, Kind},
    repository::{objects::{ObjectsRepository, ObjectOverwriteBehavior}, DeleteResult},
};
use futures::TryStreamExt;
use icu::collator::Collator;
use normalize_path::NormalizePath;
use tokio::{
    fs::{canonicalize, read_dir, remove_file, DirBuilder, File},
    io::{copy, AsyncRead},
};
use tokio_stream::wrappers::ReadDirStream;

fn kind(file_type: FileType) -> Kind {
    if file_type.is_dir() {
        Kind::Container
    } else {
        Kind::Object
    }
}

#[derive(Clone, Constructor)]
pub struct FilesystemObjectsRepository {
    collator: Arc<Collator>,
    root_dir: String,
}

impl FilesystemObjectsRepository {
    fn normalize(&self, path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        let path = path.as_ref().strip_prefix("/").context("absolute path required")?;
        let subpath = path.try_normalize().context("invalid path")?;
        let fullpath = Path::new(&self.root_dir).join(subpath);
        Ok(fullpath)
    }
}

impl ObjectsRepository for FilesystemObjectsRepository {
    type Read = File;

    async fn put<R>(&self, path: &str, mut content: R, overwrite: ObjectOverwriteBehavior) -> anyhow::Result<()>
    where
        R: AsyncRead + Send + Unpin,
    {
        let path = self.normalize(path)?;

        if let Some(parent) = path.parent() {
            DirBuilder::new()
                .mode(0o0755)
                .recursive(true)
                .create(parent)
                .await
                .with_context(|| format!("failed to create directories: {:?}", parent))?;
        }

        let result = File::options()
            .create_new(overwrite.is_denied())
            .write(true)
            .open(&path)
            .await;

        let mut file = match result {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::AlreadyExists => return Err(e).context("file already exists"),
            Err(e) => return Err(e).with_context(|| format!("failed to open file: {:?}", path)),
        };

        copy(&mut content, &mut file).await?;
        Ok(())
    }

    async fn get(&self, path: &str) -> anyhow::Result<Self::Read> {
        let path = self.normalize(path)?;
        let file = File::open(&path)
            .await
            .with_context(|| format!("failed to open file: {:?}", path))?;

        Ok(file)
    }

    async fn list(&self, prefix: &str) -> anyhow::Result<Vec<Entry>> {
        let path = self.normalize(prefix)?;
        let readdir = read_dir(&path)
            .await
            .with_context(|| format!("failed to read dir: {:?}", path))?;

        let canonical_path = canonicalize(&path)
            .await
            .with_context(|| format!("failed to canonicalize: {:?}", path))?
            .strip_prefix(&self.root_dir)?
            .to_owned();

        let mut entries: Vec<_> = ReadDirStream::new(readdir)
            .try_filter_map(|d| {
                let canonical_path = canonical_path.clone();
                async move {
                    let file_name = d.file_name();
                    let file_type = d.file_type().await?;
                    let file_path = Path::new("/").join(canonical_path).join(&file_name);
                    let entry = Entry::new(
                        file_name.to_string_lossy().into_owned(),
                        file_path.to_string_lossy().into_owned(),
                        kind(file_type),
                    );
                    Ok(Some(entry))
                }
            })
            .try_collect()
            .await?;

        entries.sort_by(|a, b| self.collator.compare(&a.name, &b.name));
        Ok(entries)
    }

    async fn delete(&self, path: &str) -> anyhow::Result<DeleteResult> {
        let path = self.normalize(path)?;

        match remove_file(&path).await {
            Ok(()) => Ok(DeleteResult::Deleted(1)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(DeleteResult::NotFound),
            Err(e) => Err(e).with_context(|| format!("failed to remove file: {:?}", path)),
        }
    }
}
