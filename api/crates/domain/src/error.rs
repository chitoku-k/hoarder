use std::fmt::{self, Write};

use indenter::indented;

use crate::entity::{
    external_services::ExternalServiceId,
    media::MediumId,
    objects::Entry,
    replicas::{ReplicaId, ThumbnailId},
    sources::SourceId,
    tag_types::TagTypeId,
    tags::TagId,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,
    #[source]
    error: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error {
            kind,
            error: Some(error.into()),
        }
    }

    pub fn other<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::new(ErrorKind::Other, error)
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn error(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.error.as_deref()
    }

    pub fn into_inner(self) -> (ErrorKind, Option<Box<dyn std::error::Error + Send + Sync + 'static>>) {
        (self.kind, self.error)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            kind,
            error: None,
        }
    }
}

struct Chain<'a>(Option<&'a dyn std::error::Error>);

impl<'a> Iterator for Chain<'a> {
    type Item = &'a (dyn std::error::Error);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(error) = self.0 {
            self.0 = error.source();
            Some(error)
        } else {
            None
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source = match (&self.kind, &self.error) {
            (ErrorKind::Other, Some(source)) => {
                write!(f, "{}", source)?;
                source.source()
            },
            (kind, error) => {
                writeln!(f, "{}", kind)?;
                writeln!(f, "Details:")?;
                write!(indented(f).with_str("      "), "{:#?}", kind)?;
                error.as_deref().map(|e| e as _)
            },
        };

        if let Some(source) = source {
            write!(f, "\nCaused by:")?;

            let multiple = source.source().is_some();
            for (i, error) in Chain(Some(source)).enumerate() {
                writeln!(f)?;

                if multiple {
                    write!(indented(f).ind(i), "{}", error)?;
                } else {
                    write!(indented(f).with_str("      "), "{}", error)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error("the external service was not found")]
    ExternalServiceNotFound { id: ExternalServiceId },

    #[error("the external service with the same slug is already registered")]
    ExternalServiceSlugDuplicate { slug: String },

    #[error("the external service url pattern is invalid")]
    ExternalServiceUrlPatternInvalid { url_pattern: String, description: Option<String> },

    #[error("the medium was not found")]
    MediumNotFound { id: MediumId },

    #[error("the medium replica was unable to be decoded")]
    MediumReplicaDecodeFailed,

    #[error("the medium replica was unable to be encoded")]
    MediumReplicaEncodeFailed,

    #[error("the medium replica was unable to be read")]
    MediumReplicaReadFailed,

    #[error("the medium replica is in unsupported format")]
    MediumReplicaUnsupported,

    #[error("the medium replicas do not match")]
    MediumReplicasNotMatch { medium_id: MediumId, expected_replicas: Vec<ReplicaId>, actual_replicas: Vec<ReplicaId> },

    #[error("the medium source was not found")]
    MediumSourceNotFound { id: MediumId },

    #[error("the medium tag was not found")]
    MediumTagNotFound { id: MediumId },

    #[error("the object with the same path already exists")]
    ObjectAlreadyExists { url: String, entry: Option<Box<Entry>> },

    #[error("the object was unable to be deleted")]
    ObjectDeleteFailed { url: String },

    #[error("the object was unable to be gotten")]
    ObjectGetFailed { url: String },

    #[error("the objects were unable to be listed")]
    ObjectListFailed { url: String },

    #[error("the object was not found")]
    ObjectNotFound { url: String },

    #[error("the object path is invalid")]
    ObjectPathInvalid,

    #[error("the object was unable to be put")]
    ObjectPutFailed { url: String },

    #[error("the object URL is invalid")]
    ObjectUrlInvalid { url: String },

    #[error("the object URL is unsupported")]
    ObjectUrlUnsupported { url: String },

    #[error("the replica was not found")]
    ReplicaNotFound { id: ReplicaId },

    #[error("the replica with the original_url was not found")]
    ReplicaNotFoundByUrl { original_url: String },

    #[error("the replica with the same original_url is already registered")]
    ReplicaOriginalUrlDuplicate { original_url: String, entry: Option<Box<Entry>> },

    #[error("the source with the same metadata is already registered")]
    SourceMetadataDuplicate { id: Option<SourceId> },

    #[error("the source metadata is invalid")]
    SourceMetadataInvalid,

    #[error("the source metadata does not match with external service")]
    SourceMetadataNotMatch { kind: String },

    #[error("the source was not found")]
    SourceNotFound { id: SourceId },

    #[error("the tag root cannot be attached")]
    TagAttachingRoot,

    #[error("the tag cannot be attached to its descendants")]
    TagAttachingToDescendant { id: TagId },

    #[error("the tag cannot be attached to itself")]
    TagAttachingToItself { id: TagId },

    #[error("the tag has {}", if .children.len() == 1 { "a child" } else { "children" })]
    TagChildrenExist { id: TagId, children: Vec<TagId> },

    #[error("the tag root cannot be deleted")]
    TagDeletingRoot,

    #[error("the tag root cannot be detached")]
    TagDetachingRoot,

    #[error("the tag was not found")]
    TagNotFound { id: TagId },

    #[error("the tag type with the same slug is already registered")]
    TagTypeSlugDuplicate { slug: String },

    #[error("the tag type was not found")]
    TagTypeNotFound { id: TagTypeId },

    #[error("the tag root cannot be updated")]
    TagUpdatingRoot,

    #[error("the thumbnail was not found")]
    ThumbnailNotFound { id: ThumbnailId },

    #[error("other error")]
    Other,
}
