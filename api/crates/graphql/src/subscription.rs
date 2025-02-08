use std::marker::PhantomData;

use async_graphql::{Context, Subscription};
use domain::service::media::MediaServiceInterface;
use futures::{future::ready, Stream, StreamExt, TryStreamExt};
use tracing_futures::Instrument;
use uuid::Uuid;

use crate::{error::{Error, Result}, media::Medium, tags::get_tag_depth};

#[derive(Default)]
pub struct Subscription<MediaService> {
    media_service: PhantomData<fn() -> MediaService>,
}

impl<MediaService> Subscription<MediaService> {
    pub fn new() -> Self {
        Self {
            media_service: PhantomData,
        }
    }
}

#[Subscription]
impl<MediaService> Subscription<MediaService>
where
    MediaService: MediaServiceInterface,
{
    /// Subscribes to a medium.
    #[tracing::instrument(skip_all)]
    async fn medium<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "The ID of the Medium object.")]
        id: Uuid,
    ) -> Result<impl Stream<Item = Medium> + 'a> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let node = ctx.look_ahead();

        let tags = node.field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = node.field("replicas").exists();
        let sources = node.field("sources").exists();

        let stream = media_service
            .watch_medium_by_id(id.into(), tag_depth, replicas, sources)
            .await?
            .map_err(Error::from)
            .and_then(|medium| ready(medium.try_into()))
            .filter_map(|result| ready(result.ok()))
            .in_current_span();

        Ok(stream)
    }
}
