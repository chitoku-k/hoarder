use async_graphql::{value, ErrorExtensions};
use domain::entity::{
    external_services::ExternalServiceId,
    media::MediumId,
    replicas::{ReplicaId, ThumbnailId},
    sources::SourceId,
    tag_types::TagTypeId,
    tags::TagId,
};
use serde::Serialize;

use crate::objects::ObjectEntry;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub(crate) struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            kind,
        }
    }
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
    ObjectAlreadyExists { url: String, entry: Option<Box<ObjectEntry>> },

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
    ReplicaOriginalUrlDuplicate { original_url: String, entry: Option<Box<ObjectEntry>> },

    #[error("the source with the same metadata is already registered")]
    SourceMetadataDuplicate { id: Option<SourceId> },

    #[error("the source metadata is invalid")]
    SourceMetadataInvalid,

    #[error("the source metadata does not match with external service")]
    SourceMetadataNotMatch { kind: String },

    #[error("the source was not found")]
    SourceNotFound { id: SourceId },

    #[error("the tag cannot be attached to its descendants")]
    TagAttachingToDescendant { id: TagId },

    #[error("the tag cannot be attached to itself")]
    TagAttachingToItself { id: TagId },

    #[error("the tag has one or more children")]
    TagChildrenExist { id: TagId, children: Vec<TagId> },

    #[error("the tag was not found")]
    TagNotFound { id: TagId },

    #[error("the tag type was not found")]
    TagTypeNotFound { id: TagTypeId },

    #[error("the tag type with the same slug is already registered")]
    TagTypeSlugDuplicate { slug: String },

    #[error("the thumbnail was not found")]
    ThumbnailNotFound { id: ThumbnailId },

    #[error("internal server error")]
    InternalServerError,

    #[error("server error")]
    GraphQLError(async_graphql::Error),
}

impl From<domain::error::Error> for Error {
    fn from(e: domain::error::Error) -> Self {
        let (kind, ..) = e.into_inner();
        Self {
            kind: kind.into(),
        }
    }
}

impl From<domain::error::ErrorKind> for ErrorKind {
    fn from(kind: domain::error::ErrorKind) -> Self {
        use domain::error::ErrorKind::*;
        match kind {
            ExternalServiceNotFound { id } => ErrorKind::ExternalServiceNotFound { id },
            ExternalServiceSlugDuplicate { slug } => ErrorKind::ExternalServiceSlugDuplicate { slug },
            ExternalServiceUrlPatternInvalid { url_pattern, description } => ErrorKind::ExternalServiceUrlPatternInvalid { url_pattern, description },
            MediumNotFound { id } => ErrorKind::MediumNotFound { id },
            MediumReplicaDecodeFailed => ErrorKind::MediumReplicaDecodeFailed,
            MediumReplicaEncodeFailed => ErrorKind::MediumReplicaEncodeFailed,
            MediumReplicaReadFailed => ErrorKind::MediumReplicaReadFailed,
            MediumReplicaUnsupported => ErrorKind::MediumReplicaUnsupported,
            MediumReplicasNotMatch { medium_id, expected_replicas, actual_replicas } => ErrorKind::MediumReplicasNotMatch { medium_id, expected_replicas, actual_replicas },
            MediumSourceNotFound { id } => ErrorKind::MediumSourceNotFound { id },
            MediumTagNotFound { id } => ErrorKind::MediumTagNotFound { id },
            ObjectAlreadyExists { url, entry } => ErrorKind::ObjectAlreadyExists {
                url,
                entry: entry.map(|e| Box::new(ObjectEntry::from(*e))),
            },
            ObjectDeleteFailed { url, .. } => ErrorKind::ObjectDeleteFailed { url },
            ObjectGetFailed { url, .. } => ErrorKind::ObjectGetFailed { url },
            ObjectListFailed { url, .. } => ErrorKind::ObjectListFailed { url },
            ObjectNotFound { url } => ErrorKind::ObjectNotFound { url },
            ObjectPathInvalid => ErrorKind::ObjectPathInvalid,
            ObjectPutFailed { url, .. } => ErrorKind::ObjectPutFailed { url },
            ObjectUrlInvalid { url } => ErrorKind::ObjectUrlInvalid { url },
            ObjectUrlUnsupported { url } => ErrorKind::ObjectUrlUnsupported { url },
            ReplicaNotFound { id } => ErrorKind::ReplicaNotFound { id },
            ReplicaNotFoundByUrl { original_url } => ErrorKind::ReplicaNotFoundByUrl { original_url },
            ReplicaOriginalUrlDuplicate { original_url, entry } => ErrorKind::ReplicaOriginalUrlDuplicate {
                original_url,
                entry: entry.map(|e| Box::new(ObjectEntry::from(*e))),
            },
            SourceMetadataDuplicate { id } => ErrorKind::SourceMetadataDuplicate { id },
            SourceMetadataInvalid => ErrorKind::SourceMetadataInvalid,
            SourceMetadataNotMatch { kind } => ErrorKind::SourceMetadataNotMatch { kind },
            SourceNotFound { id } => ErrorKind::SourceNotFound { id },
            TagAttachingRoot | TagDeletingRoot | TagDetachingRoot | TagUpdatingRoot => ErrorKind::TagNotFound { id: TagId::default() },
            TagAttachingToDescendant { id } => ErrorKind::TagAttachingToDescendant { id },
            TagAttachingToItself { id } => ErrorKind::TagAttachingToItself { id },
            TagChildrenExist { id, children } => ErrorKind::TagChildrenExist { id, children },
            TagNotFound { id } => ErrorKind::TagNotFound { id },
            TagTypeSlugDuplicate { slug } => ErrorKind::TagTypeSlugDuplicate { slug },
            TagTypeNotFound { id } => ErrorKind::TagTypeNotFound { id },
            ThumbnailNotFound { id } => ErrorKind::ThumbnailNotFound { id },
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
