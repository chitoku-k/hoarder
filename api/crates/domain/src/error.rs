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

#[derive(Debug, thiserror::Error)]
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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error("the external service with the same slug is already registered")]
    ExternalServiceDuplicateSlug { slug: String },

    #[error("the external service was not found")]
    ExternalServiceNotFound { id: ExternalServiceId },

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
    ObjectAlreadyExists { path: String, entry: Option<Box<Entry>> },

    #[error("the object was unable to be deleted")]
    ObjectDeleteFailed { path: String },

    #[error("the object was unable to be gotten")]
    ObjectGetFailed { path: String },

    #[error("the objects were unable to be listed")]
    ObjectListFailed { path: String },

    #[error("the object was not found")]
    ObjectNotFound { path: String },

    #[error("the object path is invalid")]
    ObjectPathInvalid { path: String },

    #[error("the object was unable to be put")]
    ObjectPutFailed { path: String },

    #[error("the object URL is unsupported")]
    ObjectUrlUnsupported { url: String },

    #[error("the replica was not found")]
    ReplicaNotFound { id: ReplicaId },

    #[error("the replica with the original_url was not found")]
    ReplicaNotFoundByUrl { original_url: String },

    #[error("the replica with the same original_url is already registered")]
    ReplicaDuplicateOriginalUrl { original_url: String },

    #[error("the source metadata is invalid")]
    SourceMetadataInvalid,

    #[error("the source metadata does not match with external service")]
    SourceMetadataNotMatch { slug: String },

    #[error("the source was not found")]
    SourceNotFound { id: SourceId },

    #[error("the tag root cannot be attached")]
    TagAttachingRoot,

    #[error("the tag cannot be attached to itself")]
    TagAttachingToItself { id: TagId },

    #[error("the tag cannot be attached to its descendants")]
    TagAttachingToDescendant { id: TagId },

    #[error("the tag has {}", if .children.len() == 1 { "a child" } else { "children" })]
    TagChildrenExist { id: TagId, children: Vec<TagId> },

    #[error("the tag root cannot be deleted")]
    TagDeletingRoot,

    #[error("the tag root cannot be detached")]
    TagDetachingRoot,

    #[error("the tag was not found")]
    TagNotFound { id: TagId },

    #[error("the tag root cannot be updated")]
    TagUpdatingRoot,

    #[error("the tag type with the same slug is already registered")]
    TagTypeDuplicateSlug { slug: String },

    #[error("the tag type was not found")]
    TagTypeNotFound { id: TagTypeId },

    #[error("the thumbnail was not found")]
    ThumbnailNotFound { id: ThumbnailId },

    #[error("other error")]
    Other,
}
