use std::sync::Arc;

use application::service::{
    media::MediaURLFactoryInterface,
    thumbnails::ThumbnailURLFactoryInterface,
};
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject, Upload};
use chrono::{DateTime, Utc};
use domain::{entity::replicas, service::media::MediumOverwriteBehavior};
use uuid::Uuid;

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Replica {
    id: Uuid,
    display_order: u32,
    thumbnail: Option<Thumbnail>,
    original_url: String,
    mime_type: String,
    width: u32,
    height: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, InputObject)]
pub struct ReplicaInput {
    file: Upload,
    overwrite: bool,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Thumbnail {
    id: Uuid,
    width: u32,
    height: u32,
    created_at: DateTime<Utc>,
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
            width: replica.size.width,
            height: replica.size.height,
            created_at: replica.created_at,
            updated_at: replica.updated_at,
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
    async fn url(&self, ctx: &Context<'_>) -> Option<String> {
        let media_url_factory = ctx.data_unchecked::<Arc<dyn MediaURLFactoryInterface>>();
        media_url_factory.public_url(&self.original_url)
    }
}

#[ComplexObject]
impl Thumbnail {
    async fn url(&self, ctx: &Context<'_>) -> String {
        let thumbnail_url_factory = ctx.data_unchecked::<Arc<dyn ThumbnailURLFactoryInterface>>();
        thumbnail_url_factory.get(self.id.into())
    }
}
