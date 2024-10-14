use std::marker::PhantomData;

use async_graphql::{Context, Object, SimpleObject};
use chrono::{DateTime, FixedOffset};
use domain::{
    entity::objects::{EntryUrl, EntryUrlPath},
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

#[derive(Default)]
pub struct Mutation<ExternalServicesService, MediaService, TagsService> {
    external_services_service: PhantomData<fn() -> ExternalServicesService>,
    media_service: PhantomData<fn() -> MediaService>,
    tags_service: PhantomData<fn() -> TagsService>,
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
            Ok(MediumSource::Content(EntryUrlPath::from(filename), content, overwrite))
        },
    }
}

impl<ExternalServicesService, MediaService, TagsService> Mutation<ExternalServicesService, MediaService, TagsService> {
    pub fn new() -> Self {
        Self {
            external_services_service: PhantomData,
            media_service: PhantomData,
            tags_service: PhantomData,
        }
    }
}

#[Object]
impl<ExternalServicesService, MediaService, TagsService> Mutation<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
{
    async fn create_external_service(&self, ctx: &Context<'_>, slug: String, kind: String, name: String, base_url: Option<String>) -> Result<ExternalService> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();

        let service = external_services_service.create_external_service(&slug, &kind, &name, base_url.as_deref()).await?;
        Ok(service.into())
    }

    async fn update_external_service(&self, ctx: &Context<'_>, id: Uuid, slug: Option<String>, name: Option<String>, base_url: Option<String>) -> Result<ExternalService> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();

        let base_url = match &base_url {
            Some(base_url) if base_url.is_empty() => Some(None),
            Some(base_url) => Some(Some(base_url.as_str())),
            None => None,
        };

        let service = external_services_service.update_external_service_by_id(id.into(), slug.as_deref(), name.as_deref(), base_url).await?;
        Ok(service.into())
    }

    async fn delete_external_service(&self, ctx: &Context<'_>, id: Uuid) -> Result<DeleteResult> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();

        let result = external_services_service.delete_external_service_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn create_medium(
        &self,
        ctx: &Context<'_>,
        source_ids: Option<Vec<Uuid>>,
        created_at: Option<DateTime<FixedOffset>>,
        tag_ids: Option<Vec<TagTagTypeInput>>,
    ) -> Result<Medium> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let tags = ctx.look_ahead().field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let sources = ctx.look_ahead().field("sources").exists();

        let source_ids: Map<_, _, _> = source_ids.unwrap_or_default().into_iter().map(Into::into);
        let tag_tag_type_ids: Map<_, _, _> = tag_ids.unwrap_or_default().into_iter().map(Into::into);

        let created_at = created_at.map(Into::into);

        let medium = media_service.create_medium(source_ids, created_at, tag_tag_type_ids, tag_depth, sources).await?;
        let medium = medium.try_into().map_err(Error::new)?;
        Ok(medium)
    }

    async fn create_replica(&self, ctx: &Context<'_>, medium_id: Uuid, original_url: Option<String>, upload: Option<ReplicaInput>) -> Result<Replica> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let medium_source = create_medium_source(ctx, original_url, upload).await?;
        let replica = media_service.create_replica(medium_id.into(), medium_source).await?;
        Ok(replica.into())
    }

    async fn create_source(&self, ctx: &Context<'_>, external_service_id: Uuid, external_metadata: ExternalMetadata) -> Result<Source> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let external_service_id = external_service_id.into();
        let external_metadata = external_metadata.try_into().map_err(Error::new)?;

        let source = media_service.create_source(external_service_id, external_metadata).await?;
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
        let media_service = ctx.data_unchecked::<MediaService>();

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

        let medium = media_service.update_medium_by_id(
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
        let media_service = ctx.data_unchecked::<MediaService>();

        let medium_source = create_medium_source(ctx, original_url, upload).await?;
        let replica = media_service.update_replica_by_id(id.into(), medium_source).await?;
        Ok(replica.into())
    }

    async fn update_source(&self, ctx: &Context<'_>, id: Uuid, external_service_id: Option<Uuid>, external_metadata: Option<ExternalMetadata>) -> Result<Source> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let external_service_id = external_service_id.map(Into::into);
        let external_metadata = external_metadata.map(TryInto::try_into).transpose().map_err(Error::new)?;

        let source = media_service.update_source_by_id(id.into(), external_service_id, external_metadata).await?;
        let source = source.try_into().map_err(Error::new)?;
        Ok(source)
    }

    async fn delete_medium(&self, ctx: &Context<'_>, id: Uuid, delete_objects: Option<bool>) -> Result<DeleteResult> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let delete_objects = delete_objects.unwrap_or_default();

        let result = media_service.delete_medium_by_id(id.into(), delete_objects).await?;
        Ok(result.into())
    }

    async fn delete_replica(&self, ctx: &Context<'_>, id: Uuid, delete_object: Option<bool>) -> Result<DeleteResult> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let delete_object = delete_object.unwrap_or_default();

        let result = media_service.delete_replica_by_id(id.into(), delete_object).await?;
        Ok(result.into())
    }

    async fn delete_source(&self, ctx: &Context<'_>, id: Uuid) -> Result<DeleteResult> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let result = media_service.delete_source_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn create_tag(&self, ctx: &Context<'_>, name: String, kana: String, aliases: Option<Vec<String>>, parent_id: Option<Uuid>) -> Result<Tag> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let depth = get_tag_depth(&ctx.look_ahead());
        let aliases = aliases.unwrap_or_default();

        let tag = tags_service.create_tag(&name, &kana, &aliases, parent_id.map(Into::into), depth).await?;
        Ok(tag.into())
    }

    async fn create_tag_type(&self, ctx: &Context<'_>, slug: String, name: String, kana: String) -> Result<TagType> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let tag_type = tags_service.create_tag_type(&slug, &name, &kana).await?;
        Ok(tag_type.into())
    }

    async fn update_tag(&self, ctx: &Context<'_>, id: Uuid, name: Option<String>, kana: Option<String>, add_aliases: Option<Vec<String>>, remove_aliases: Option<Vec<String>>) -> Result<Tag> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let depth = get_tag_depth(&ctx.look_ahead());
        let add_aliases = add_aliases.unwrap_or_default();
        let remove_aliases = remove_aliases.unwrap_or_default();

        let tag = tags_service.update_tag_by_id(id.into(), name, kana, add_aliases, remove_aliases, depth).await?;
        Ok(tag.into())
    }

    async fn update_tag_type(&self, ctx: &Context<'_>, id: Uuid, slug: Option<String>, name: Option<String>, kana: Option<String>) -> Result<TagType> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let tag_type = tags_service.update_tag_type_by_id(id.into(), slug.as_deref(), name.as_deref(), kana.as_deref()).await?;
        Ok(tag_type.into())
    }

    async fn attach_tag(&self, ctx: &Context<'_>, id: Uuid, parent_id: Uuid) -> Result<Tag> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = tags_service.attach_tag_by_id(id.into(), parent_id.into(), depth).await?;
        Ok(tag.into())
    }

    async fn detach_tag(&self, ctx: &Context<'_>, id: Uuid) -> Result<Tag> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = tags_service.detach_tag_by_id(id.into(), depth).await?;
        Ok(tag.into())
    }

    async fn delete_tag(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        #[graphql(default = false)]
        recursive: bool,
    ) -> Result<DeleteResult> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let result = tags_service.delete_tag_by_id(id.into(), recursive).await?;
        Ok(result.into())
    }

    async fn delete_tag_type(&self, ctx: &Context<'_>, id: Uuid) -> Result<DeleteResult> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let result = tags_service.delete_tag_type_by_id(id.into()).await?;
        Ok(result.into())
    }
}
