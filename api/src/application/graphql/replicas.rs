use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::{
    application::thumbnails::ThumbnailURLFactory,
    domain::entity::replicas,
};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Replica {
    id: Uuid,
    display_order: Option<u32>,
    #[graphql(skip)]
    has_thumbnail: bool,
    original_url: String,
    mime_type: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<replicas::Replica> for Replica {
    fn from(replica: replicas::Replica) -> Self {
        Self {
            id: *replica.id,
            display_order: replica.display_order,
            has_thumbnail: replica.has_thumbnail,
            original_url: replica.original_url,
            mime_type: replica.mime_type,
            created_at: replica.created_at,
            updated_at: replica.updated_at,
        }
    }
}

#[ComplexObject]
impl Replica {
    async fn thumbnail_url(&self, ctx: &Context<'_>) -> Option<String> {
        self.has_thumbnail.then(|| ctx.data_unchecked::<ThumbnailURLFactory>().url(&self.id.into()))
    }
}
