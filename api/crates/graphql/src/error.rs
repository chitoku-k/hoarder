use async_graphql::{value, ErrorExtensions};
use serde::Serialize;
use uuid::Uuid;

use crate::objects::ObjectEntry;

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Error {
    kind: ErrorKind,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "code", content = "data", rename_all = "SCREAMING_SNAKE_CASE", rename_all_fields = "camelCase")]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ErrorKind {
    #[error("the argument is required")]
    ArgumentRequired { one_of: Vec<&'static str> },

    #[error("the arguments are mutually exclusive")]
    ArgumentsMutuallyExclusive { arguments: Vec<&'static str> },

    #[error("the cursor is invalid")]
    CursorInvalid,

    #[error("the external service was not found")]
    ExternalServiceNotFound { id: Uuid },

    #[error("the external service with the same slug is already registered")]
    ExternalServiceSlugDuplicate { slug: String },

    #[error("the medium was not found")]
    MediumNotFound { id: Uuid },

    #[error("the medium replica was unable to be decoded")]
    MediumReplicaDecodeFailed,

    #[error("the medium replica was unable to be encoded")]
    MediumReplicaEncodeFailed,

    #[error("the medium replica was unable to be read")]
    MediumReplicaReadFailed,

    #[error("the medium replica is in unsupported format")]
    MediumReplicaUnsupported,

    #[error("the medium replicas do not match")]
    MediumReplicasNotMatch { medium_id: Uuid, expected_replicas: Vec<Uuid>, actual_replicas: Vec<Uuid> },

    #[error("the medium source was not found")]
    MediumSourceNotFound { id: Uuid },

    #[error("the medium tag was not found")]
    MediumTagNotFound { id: Uuid },

    #[error("the object with the same path already exists")]
    ObjectAlreadyExists { path: String, entry: Option<Box<ObjectEntry>> },

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
    ReplicaNotFound { id: Uuid },

    #[error("the replica with the original_url was not found")]
    ReplicaNotFoundByUrl { original_url: String },

    #[error("the replica with the same original_url is already registered")]
    ReplicaOriginalUrlDuplicate { original_url: String },

    #[error("the source with the same metadata is already registered")]
    SourceMetadataDuplicate { id: Option<Uuid> },

    #[error("the source metadata is invalid")]
    SourceMetadataInvalid,

    #[error("the source metadata does not match with external service")]
    SourceMetadataNotMatch { slug: String },

    #[error("the source was not found")]
    SourceNotFound { id: Uuid },

    #[error("the tag cannot be attached to its descendants")]
    TagAttachingToDescendant { id: Uuid },

    #[error("the tag cannot be attached to itself")]
    TagAttachingToItself { id: Uuid },

    #[error("the tag has one or more children")]
    TagChildrenExist { id: Uuid, children: Vec<Uuid> },

    #[error("the tag was not found")]
    TagNotFound { id: Uuid },

    #[error("the tag type was not found")]
    TagTypeNotFound { id: Uuid },

    #[error("the tag type with the same slug is already registered")]
    TagTypeSlugDuplicate { slug: String },

    #[error("the thumbnail was not found")]
    ThumbnailNotFound { id: Uuid },

    #[error("internal server error")]
    InternalServerError,

    #[error("server error")]
    GraphQLError(async_graphql::Error),
}

impl Error {
    pub fn new<K>(kind: K) -> Error
    where
        K: Into<ErrorKind>,
    {
        Self {
            kind: kind.into(),
        }
    }
}

impl From<domain::error::Error> for Error {
    fn from(e: domain::error::Error) -> Self {
        let (kind, ..) = e.into_inner();
        Self {
            kind: kind.into(),
        }
    }
}

impl From<domain::error::Error> for ErrorKind {
    fn from(e: domain::error::Error) -> Self {
        let (kind, ..) = e.into_inner();
        kind.into()
    }
}

impl From<domain::error::ErrorKind> for ErrorKind {
    fn from(kind: domain::error::ErrorKind) -> Self {
        use domain::error::ErrorKind::*;
        match kind {
            ExternalServiceNotFound { id } => ErrorKind::ExternalServiceNotFound { id: *id },
            ExternalServiceSlugDuplicate { slug } => ErrorKind::ExternalServiceSlugDuplicate { slug },
            MediumNotFound { id } => ErrorKind::MediumNotFound { id: *id },
            MediumReplicaDecodeFailed => ErrorKind::MediumReplicaDecodeFailed,
            MediumReplicaEncodeFailed => ErrorKind::MediumReplicaEncodeFailed,
            MediumReplicaReadFailed => ErrorKind::MediumReplicaReadFailed,
            MediumReplicaUnsupported => ErrorKind::MediumReplicaUnsupported,
            MediumReplicasNotMatch { medium_id, expected_replicas, actual_replicas } => ErrorKind::MediumReplicasNotMatch {
                medium_id: *medium_id,
                expected_replicas: expected_replicas.into_iter().map(|r| *r).collect(),
                actual_replicas: actual_replicas.into_iter().map(|r| *r).collect(),
            },
            MediumSourceNotFound { id } => ErrorKind::MediumSourceNotFound { id: *id },
            MediumTagNotFound { id } => ErrorKind::MediumTagNotFound { id: *id },
            ObjectAlreadyExists { path, entry } => ErrorKind::ObjectAlreadyExists {
                path,
                entry: entry.map(|e| Box::new(ObjectEntry::from(*e))),
            },
            ObjectDeleteFailed { path, .. } => ErrorKind::ObjectDeleteFailed { path },
            ObjectGetFailed { path, .. } => ErrorKind::ObjectGetFailed { path },
            ObjectListFailed { path, .. } => ErrorKind::ObjectListFailed { path },
            ObjectNotFound { path } => ErrorKind::ObjectNotFound { path },
            ObjectPathInvalid { path } => ErrorKind::ObjectPathInvalid { path },
            ObjectPutFailed { path, .. } => ErrorKind::ObjectPutFailed { path },
            ObjectUrlUnsupported { url } => ErrorKind::ObjectUrlUnsupported { url },
            ReplicaNotFound { id } => ErrorKind::ReplicaNotFound { id: *id },
            ReplicaNotFoundByUrl { original_url } => ErrorKind::ReplicaNotFoundByUrl { original_url },
            ReplicaOriginalUrlDuplicate { original_url } => ErrorKind::ReplicaOriginalUrlDuplicate { original_url },
            SourceMetadataDuplicate { id } => ErrorKind::SourceMetadataDuplicate { id: id.map(|id| *id) },
            SourceMetadataInvalid => ErrorKind::SourceMetadataInvalid,
            SourceMetadataNotMatch { slug } => ErrorKind::SourceMetadataNotMatch { slug },
            SourceNotFound { id } => ErrorKind::SourceNotFound { id: *id },
            TagAttachingRoot | TagDeletingRoot | TagDetachingRoot | TagUpdatingRoot => ErrorKind::TagNotFound { id: Uuid::nil() },
            TagAttachingToDescendant { id } => ErrorKind::TagAttachingToDescendant { id: *id },
            TagAttachingToItself { id } => ErrorKind::TagAttachingToItself { id: *id },
            TagChildrenExist { id, children } => ErrorKind::TagChildrenExist {
                id: *id,
                children: children.into_iter().map(|c| *c).collect(),
            },
            TagNotFound { id } => ErrorKind::TagNotFound { id: *id },
            TagTypeSlugDuplicate { slug } => ErrorKind::TagTypeSlugDuplicate { slug },
            TagTypeNotFound { id } => ErrorKind::TagTypeNotFound { id: *id },
            ThumbnailNotFound { id } => ErrorKind::ThumbnailNotFound { id: *id },
            _ => ErrorKind::InternalServerError,
        }
    }
}

impl From<Error> for async_graphql::Error {
    fn from(e: Error) -> Self {
        e.extend()
    }
}

impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        match &self.kind {
            ErrorKind::GraphQLError(e) => e.clone(),
            kind => {
                async_graphql::Error::new(kind.to_string())
                    .extend_with(|_, e| e.set("details", value!(kind)))
            },
        }
    }
}
