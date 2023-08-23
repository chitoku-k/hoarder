use std::io::Write;

use derive_more::Constructor;
use domain::service::{
    external_services::ExternalServicesServiceInterface,
    media::MediaServiceInterface,
    tags::TagsServiceInterface,
};
use graphql::APISchema;

#[derive(Constructor)]
pub struct PrintSchema<ExternalServicesService, MediaService, TagsService> {
    schema: APISchema<ExternalServicesService, MediaService, TagsService>,
}

impl<ExternalServicesService, MediaService, TagsService> PrintSchema<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
{
    pub fn print<W>(&self, w: &mut W) -> anyhow::Result<()>
    where
        W: Write,
    {
        write!(w, "{}", self.schema.sdl())?;

        Ok(())
    }
}
