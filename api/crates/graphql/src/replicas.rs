use std::sync::Arc;

use application::service::{
    media::MediaURLFactoryInterface,
    thumbnails::ThumbnailURLFactoryInterface,
};
use async_graphql::{ComplexObject, Context, Enum, InputObject, SimpleObject, Upload};
use chrono::{DateTime, Utc};
use domain::{entity::replicas, service::media::MediumOverwriteBehavior};
use serde::Serialize;
use uuid::Uuid;

/// A replica represents metadata and a reference to the object in the storage.
#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Replica {
    /// The ID of the Replica object.
    id: Uuid,
    /// The 1-based index of the display order in the medium.
    display_order: u32,
    /// The thumbnail of the replica.
    thumbnail: Option<Thumbnail>,
    /// The internal original URL of the replica.
    original_url: String,
    /// The MIME type of the replica. Unavailable when in process.
    mime_type: Option<String>,
    /// The width of the replica. Unavailable when in process.
    width: Option<u32>,
    /// The height of the replica. Unavailable when in process.
    height: Option<u32>,
    /// The current status of the replica.
    status: ReplicaStatus,
    /// The date at which the replica was created.
    created_at: DateTime<Utc>,
    /// The date at which the replica was updated.
    updated_at: DateTime<Utc>,
}

/// A replica status represents the current status of a replica.
#[derive(Debug, Serialize, SimpleObject)]
pub(crate) struct ReplicaStatus {
    /// The phase of the replica.
    phase: ReplicaPhase,
}

/// A replica phase represents the phase of a replica.
#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum ReplicaPhase {
    /// The replica is ready to serve.
    Ready,
    /// The replica is in process.
    Processing,
    /// The replica has an error.
    Error,
}

/// A replica input represents a file upload.
#[derive(Clone, Copy, InputObject)]
pub struct ReplicaInput {
    /// The file to upload. The name must start with a single slash `/`.
    file: Upload,
    /// Whether to overwrite the existing file.
    overwrite: bool,
}

/// A thumbnail represents a smaller version of the object that is generated from the original one.
#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Thumbnail {
    /// The ID of the Thumbnail object.
    id: Uuid,
    /// The width of the thumbnail.
    width: u32,
    /// The height of the thumbnail.
    height: u32,
    /// The date at which the thumbnail was created.
    created_at: DateTime<Utc>,
    /// The date at which the thumbnail was updated.
    updated_at: DateTime<Utc>,
}

impl From<replicas::Replica> for Replica {
    fn from(replica: replicas::Replica) -> Self {
        Self {
            id: *replica.id,
            display_order: replica.display_order,
            thumbnail: replica.thumbnail.map(Into::into),
            original_url: replica.original_url,
            mime_type: replica.mime_type,
            width: replica.size.map(|size| size.width),
            height: replica.size.map(|size| size.height),
            status: replica.status.into(),
            created_at: replica.created_at,
            updated_at: replica.updated_at,
        }
    }
}

impl From<replicas::ReplicaStatus> for ReplicaStatus {
    fn from(value: replicas::ReplicaStatus) -> Self {
        use replicas::ReplicaStatus::*;
        Self {
            phase: match value {
                Ready => ReplicaPhase::Ready,
                Processing => ReplicaPhase::Processing,
                Error => ReplicaPhase::Error,
            },
        }
    }
}

impl From<ReplicaInput> for (Upload, MediumOverwriteBehavior) {
    fn from(input: ReplicaInput) -> Self {
        let file = input.file;
        let overwrite = match input.overwrite {
            true => MediumOverwriteBehavior::Overwrite,
            false => MediumOverwriteBehavior::Fail,
        };

        (file, overwrite)
    }
}

impl From<replicas::Thumbnail> for Thumbnail {
    fn from(thumbnail: replicas::Thumbnail) -> Self {
        Self {
            id: *thumbnail.id,
            width: thumbnail.size.width,
            height: thumbnail.size.height,
            created_at: thumbnail.created_at,
            updated_at: thumbnail.updated_at,
        }
    }
}

#[ComplexObject]
impl Replica {
    /// The public URL of the replica.
    async fn url(&self, ctx: &Context<'_>) -> Option<String> {
        let media_url_factory = ctx.data_unchecked::<Arc<dyn MediaURLFactoryInterface>>();
        media_url_factory.public_url(&self.original_url)
    }
}

#[ComplexObject]
impl Thumbnail {
    /// The public URL of the thumbnail. Unavailable when in process.
    async fn url(&self, ctx: &Context<'_>) -> String {
        let thumbnail_url_factory = ctx.data_unchecked::<Arc<dyn ThumbnailURLFactoryInterface>>();
        thumbnail_url_factory.get(self.id.into())
    }
}
