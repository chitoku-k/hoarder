use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};

use cow_utils::CowUtils;
use derive_more::Display;
use domain::{entity::objects::EntryUrl, error::{Error, ErrorKind, Result}};
use normalize_path::NormalizePath;

use crate::StorageEntryUrl;

#[derive(Display)]
#[display("{_0}")]
pub(crate) struct FilesystemEntryUrl(EntryUrl, PathBuf);

impl FilesystemEntryUrl {
    pub(crate) fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path
            .as_ref()
            .to_str()
            .ok_or(ErrorKind::ObjectPathInvalid)?
            .cow_replace(MAIN_SEPARATOR_STR, "/");

        Self::try_from(EntryUrl::from_path_str(Self::URL_PREFIX, &path))
    }

    pub(crate) fn as_path(&self) -> &Path {
        self.1.as_path()
    }
}

impl TryFrom<EntryUrl> for FilesystemEntryUrl {
    type Error = Error;

    fn try_from(url: EntryUrl) -> Result<Self> {
        let path = url.to_path_string(Self::URL_PREFIX)?;
        let path = match path.strip_prefix('/') {
            Some(path) => path,
            None => return Err(ErrorKind::ObjectUrlInvalid { url: url.into_inner() })?,
        };
        let path = match PathBuf::from(path).try_normalize() {
            Some(path) => path,
            None => return Err(ErrorKind::ObjectUrlInvalid { url: url.into_inner() })?,
        };

        Ok(Self(url, path))
    }
}

impl StorageEntryUrl for FilesystemEntryUrl {
    const URL_PREFIX: &'static str = "file://";

    fn into_url(self) -> EntryUrl {
        self.0
    }
}
