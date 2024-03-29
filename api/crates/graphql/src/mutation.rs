use async_graphql::{Context, Object, SimpleObject};
use chrono::{DateTime, FixedOffset};
use derive_more::Constructor;
use domain::{
    entity::objects::{EntryPath, EntryUrl},
    repository,
    service::{
        external_services::ExternalServicesServiceInterface,
        media::{MediaServiceInterface, MediumSource},
        tags::TagsServiceInterface,
    },
};
use uuid::Uuid;

use crate::{
    error::{Error, ErrorKind, Result},
    external_services::ExternalService,
    media::Medium,
    process_upload,
    replicas::{Replica, ReplicaInput},
    sources::{ExternalMetadata, Source},
    tags::{get_tag_depth, Tag, TagTagTypeInput, TagType},
};

#[derive(SimpleObject)]
pub(crate) struct DeleteResult {
    deleted: bool,
}

impl From<repository::DeleteResult> for DeleteResult {
    fn from(delete_result: repository::DeleteResult) -> Self {
        Self {
            deleted: delete_result.is_deleted(),
        }
    }
}

#[derive(Constructor)]
pub struct Mutation<ExternalServicesService, MediaService, TagsService> {
    external_services_service: ExternalServicesService,
    media_service: MediaService,
    tags_service: TagsService,
}

type Map<T, U, V> = std::iter::Map<T, fn(U) -> V>;

async fn create_medium_source(ctx: &Context<'_>, original_url: Option<String>, upload: Option<ReplicaInput>) -> Result<MediumSource> {
    match (original_url, upload) {
        (None, None) => Err(Error::new(ErrorKind::ArgumentRequired { one_of: vec!["original_url", "upload"] })),
        (Some(_), Some(_)) => Err(Error::new(ErrorKind::ArgumentsMutuallyExclusive { arguments: vec!["original_url", "upload"] })),
        (Some(original_url), None) => Ok(MediumSource::Url(EntryUrl::from(original_url))),
        (None, Some(input)) => {
            let (file, overwrite) = input.into();
            let value = file.value(ctx).map_err(|_| Error::new(ErrorKind::InternalServerError))?;
            let filename = value.filename.clone();
            let content = process_upload(value).await?;
            Ok(MediumSource::Content(EntryPath::from(filename), content, overwrite))
        },
    }
}

#[Object]
impl<ExternalServicesService, MediaService, TagsService> Mutation<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
{
    async fn create_external_service(&self, slug: String, name: String) -> Result<ExternalService> {
        let service = self.external_services_service.create_external_service(&slug, &name).await?;
        Ok(service.into())
    }

    async fn update_external_service(&self, id: Uuid, name: Option<String>) -> Result<ExternalService> {
        let service = self.external_services_service.update_external_service_by_id(id.into(), name.as_deref()).await?;
        Ok(service.into())
    }

    async fn delete_external_service(&self, id: Uuid) -> Result<DeleteResult> {
        let result = self.external_services_service.delete_external_service_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn create_medium(
        &self,
        ctx: &Context<'_>,
        source_ids: Option<Vec<Uuid>>,
        created_at: Option<DateTime<FixedOffset>>,
        tag_ids: Option<Vec<TagTagTypeInput>>,
    ) -> Result<Medium> {
        let tags = ctx.look_ahead().field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let sources = ctx.look_ahead().field("sources").exists();

        let source_ids: Map<_, _, _> = source_ids.unwrap_or_default().into_iter().map(Into::into);
        let tag_tag_type_ids: Map<_, _, _> = tag_ids.unwrap_or_default().into_iter().map(Into::into);

        let created_at = created_at.map(Into::into);

        let medium = self.media_service.create_medium(source_ids, created_at, tag_tag_type_ids, tag_depth, sources).await?;
        let medium = medium.try_into().map_err(Error::new)?;
        Ok(medium)
    }

    async fn create_replica(&self, ctx: &Context<'_>, medium_id: Uuid, original_url: Option<String>, upload: Option<ReplicaInput>) -> Result<Replica> {
        let medium_source = create_medium_source(ctx, original_url, upload).await?;
        let replica = self.media_service.create_replica(medium_id.into(), medium_source).await?;
        Ok(replica.into())
    }

    async fn create_source(&self, external_service_id: Uuid, external_metadata: ExternalMetadata) -> Result<Source> {
        let external_service_id = external_service_id.into();
        let external_metadata = external_metadata.try_into().map_err(Error::new)?;

        let source = self.media_service.create_source(external_service_id, external_metadata).await?;
        let source = source.try_into().map_err(Error::new)?;
        Ok(source)
    }

    async fn update_medium(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        add_source_ids: Option<Vec<Uuid>>,
        remove_source_ids: Option<Vec<Uuid>>,
        add_tag_ids: Option<Vec<TagTagTypeInput>>,
        remove_tag_ids: Option<Vec<TagTagTypeInput>>,
        replica_orders: Option<Vec<Uuid>>,
        created_at: Option<DateTime<FixedOffset>>,
    ) -> Result<Medium> {
        let tags = ctx.look_ahead().field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = ctx.look_ahead().field("replicas").exists();
        let sources = ctx.look_ahead().field("sources").exists();

        let add_source_ids: Map<_, _, _> = add_source_ids.unwrap_or_default().into_iter().map(Into::into);
        let remove_source_ids: Map<_, _, _> = remove_source_ids.unwrap_or_default().into_iter().map(Into::into);

        let add_tag_tag_type_ids: Map<_, _, _> = add_tag_ids.unwrap_or_default().into_iter().map(Into::into);
        let remove_tag_tag_type_ids: Map<_, _, _> = remove_tag_ids.unwrap_or_default().into_iter().map(Into::into);

        let replica_orders: Map<_, _, _> = replica_orders.unwrap_or_default().into_iter().map(Into::into);

        let created_at = created_at.map(Into::into);

        let medium = self.media_service.update_medium_by_id(
            id.into(),
            add_source_ids,
            remove_source_ids,
            add_tag_tag_type_ids,
            remove_tag_tag_type_ids,
            replica_orders,
            created_at,
            tag_depth,
            replicas,
            sources,
        ).await?;
        let medium = medium.try_into().map_err(Error::new)?;
        Ok(medium)
    }

    async fn update_replica(&self, ctx: &Context<'_>, id: Uuid, original_url: Option<String>, upload: Option<ReplicaInput>) -> Result<Replica> {
        let medium_source = create_medium_source(ctx, original_url, upload).await?;
        let replica = self.media_service.update_replica_by_id(id.into(), medium_source).await?;
        Ok(replica.into())
    }

    async fn update_source(&self, id: Uuid, external_service_id: Option<Uuid>, external_metadata: Option<ExternalMetadata>) -> Result<Source> {
        let external_service_id = external_service_id.map(Into::into);
        let external_metadata = external_metadata.map(TryInto::try_into).transpose().map_err(Error::new)?;

        let source = self.media_service.update_source_by_id(id.into(), external_service_id, external_metadata).await?;
        let source = source.try_into().map_err(Error::new)?;
        Ok(source)
    }

    async fn delete_medium(&self, id: Uuid) -> Result<DeleteResult> {
        let result = self.media_service.delete_medium_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn delete_replica(&self, id: Uuid) -> Result<DeleteResult> {
        let result = self.media_service.delete_replica_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn delete_source(&self, id: Uuid) -> Result<DeleteResult> {
        let result = self.media_service.delete_source_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn create_tag(&self, ctx: &Context<'_>, name: String, kana: String, aliases: Option<Vec<String>>, parent_id: Option<Uuid>) -> Result<Tag> {
        let depth = get_tag_depth(&ctx.look_ahead());
        let aliases = aliases.unwrap_or_default();

        let tag = self.tags_service.create_tag(&name, &kana, &aliases, parent_id.map(Into::into), depth).await?;
        Ok(tag.into())
    }

    async fn create_tag_type(&self, slug: String, name: String) -> Result<TagType> {
        let tag_type = self.tags_service.create_tag_type(&slug, &name).await?;
        Ok(tag_type.into())
    }

    async fn update_tag(&self, ctx: &Context<'_>, id: Uuid, name: Option<String>, kana: Option<String>, add_aliases: Option<Vec<String>>, remove_aliases: Option<Vec<String>>) -> Result<Tag> {
        let depth = get_tag_depth(&ctx.look_ahead());
        let add_aliases = add_aliases.unwrap_or_default();
        let remove_aliases = remove_aliases.unwrap_or_default();

        let tag = self.tags_service.update_tag_by_id(id.into(), name, kana, add_aliases, remove_aliases, depth).await?;
        Ok(tag.into())
    }

    async fn update_tag_type(&self, id: Uuid, slug: Option<String>, name: Option<String>) -> Result<TagType> {
        let tag_type = self.tags_service.update_tag_type_by_id(id.into(), slug.as_deref(), name.as_deref()).await?;
        Ok(tag_type.into())
    }

    async fn attach_tag(&self, ctx: &Context<'_>, id: Uuid, parent_id: Uuid) -> Result<Tag> {
        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = self.tags_service.attach_tag_by_id(id.into(), parent_id.into(), depth).await?;
        Ok(tag.into())
    }

    async fn detach_tag(&self, ctx: &Context<'_>, id: Uuid) -> Result<Tag> {
        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = self.tags_service.detach_tag_by_id(id.into(), depth).await?;
        Ok(tag.into())
    }

    async fn delete_tag(
        &self,
        id: Uuid,
        #[graphql(default = false)]
        recursive: bool,
    ) -> Result<DeleteResult> {
        let result = self.tags_service.delete_tag_by_id(id.into(), recursive).await?;
        Ok(result.into())
    }

    async fn delete_tag_type(&self, id: Uuid) -> Result<DeleteResult> {
        let result = self.tags_service.delete_tag_type_by_id(id.into()).await?;
        Ok(result.into())
    }
}
