use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::{DateTime, Utc};
use domain::entity::replicas;
use thumbnails::ThumbnailURLFactory;
use uuid::Uuid;

#[derive(SimpleObject)]
pub(crate) struct Replica {
    id: Uuid,
    display_order: Option<u32>,
    thumbnail: Option<Thumbnail>,
    original_url: String,
    mime_type: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Thumbnail {
    id: Uuid,
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
            created_at: replica.created_at,
            updated_at: replica.updated_at,
        }
    }
}

impl From<replicas::Thumbnail> for Thumbnail {
    fn from(thumbnail: replicas::Thumbnail) -> Self {
        Self {
            id: *thumbnail.id,
            created_at: thumbnail.created_at,
            updated_at: thumbnail.updated_at,
        }
    }
}

#[ComplexObject]
impl Thumbnail {
    async fn url(&self, ctx: &Context<'_>) -> String {
        ctx.data_unchecked::<ThumbnailURLFactory>().url(&self.id.into())
    }
}
