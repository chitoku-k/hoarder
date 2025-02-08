use std::{io::{Read, Seek}, marker::PhantomData, sync::Arc};

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
use normalizer::NormalizerInterface;
use tokio_util::task::TaskTracker;
use uuid::Uuid;

use crate::{
    error::{Error, ErrorKind, Result},
    external_services::ExternalService,
    media::Medium,
    replicas::{Replica, ReplicaInput},
    sources::{ExternalMetadata, Source},
    tags::{get_tag_depth, Tag, TagTagTypeInput, TagType},
};

/// A delete result represents the result of deletion.
#[derive(SimpleObject)]
pub(crate) struct DeleteResult {
    /// The value indicating whether the object has been deleted.
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
pub struct Mutation<ExternalServicesService, MediaService, TagsService, Normalizer> {
    external_services_service: PhantomData<fn() -> ExternalServicesService>,
    media_service: PhantomData<fn() -> MediaService>,
    tags_service: PhantomData<fn() -> TagsService>,
    normalizer: PhantomData<fn() -> Normalizer>,
}

async fn create_medium_source(ctx: &Context<'_>, original_url: Option<String>, upload: Option<ReplicaInput>) -> Result<MediumSource<impl Read + Seek + Send + Sync + 'static>> {
    match (original_url, upload) {
        (None, None) => Err(Error::new(ErrorKind::ArgumentRequired { one_of: vec!["original_url", "upload"] })),
        (Some(_), Some(_)) => Err(Error::new(ErrorKind::ArgumentsMutuallyExclusive { arguments: vec!["original_url", "upload"] })),
        (Some(original_url), None) => Ok(MediumSource::Url(EntryUrl::from(original_url))),
        (None, Some(input)) => {
            let (file, overwrite) = input.into();
            let value = file.value(ctx).map_err(|_| Error::new(ErrorKind::InternalServerError))?;
            Ok(MediumSource::Content(EntryUrlPath::from(value.filename), value.content, overwrite))
        },
    }
}

impl<ExternalServicesService, MediaService, TagsService, Normalizer> Mutation<ExternalServicesService, MediaService, TagsService, Normalizer> {
    pub fn new() -> Self {
        Self {
            external_services_service: PhantomData,
            media_service: PhantomData,
            tags_service: PhantomData,
            normalizer: PhantomData,
        }
    }
}

#[Object]
impl<ExternalServicesService, MediaService, TagsService, Normalizer> Mutation<ExternalServicesService, MediaService, TagsService, Normalizer>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
    Normalizer: NormalizerInterface,
{
    /// Creates an external service.
    /// ### Errors
    /// * When the slug already exists, it returns an `EXTERNAL_SERVICE_SLUG_DUPLICATE` error.
    /// * When the urlPattern is invalid, it returns an `EXTERNAL_SERVICE_URL_PATTERN_INVALID` error.
    async fn create_external_service(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The short and user-friendly name that uniquely identifies the Web service.")]
        slug: String,
        #[graphql(desc = "The kind of the Web service that the object represents. See ExternalService for supported values.")]
        kind: String,
        #[graphql(desc = "The name of the Web service.")]
        name: String,
        #[graphql(desc = "The base URL of the Web service. Some services do not have the base URL.")]
        base_url: Option<String>,
        #[graphql(desc = "The regex pattern of a URL in the Web service.")]
        url_pattern: Option<String>,
    ) -> Result<ExternalService> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();
        let normalizer = ctx.data_unchecked::<Arc<Normalizer>>();

        let slug = normalizer.normalize(slug);
        let kind = normalizer.normalize(kind);
        let name = normalizer.normalize(name);

        let service = external_services_service.create_external_service(&slug, &kind, &name, base_url.as_deref(), url_pattern.as_deref()).await?;
        Ok(service.into())
    }

    /// Updates an external service. Only non-null fields will be updated.
    /// ### Errors
    /// * When the slug already exists, it returns an `EXTERNAL_SERVICE_SLUG_DUPLICATE` error.
    /// * When the urlPattern is invalid, it returns an `EXTERNAL_SERVICE_URL_PATTERN_INVALID` error.
    async fn update_external_service(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the ExternalService object.")]
        id: Uuid,
        #[graphql(desc = "The short and user-friendly name that uniquely identifies the Web service.")]
        slug: Option<String>,
        #[graphql(desc = "The name of the Web service.")]
        name: Option<String>,
        #[graphql(desc = "The base URL of the Web service. Some services do not have the base URL. Pass an empty value to remove.")]
        base_url: Option<String>,
        #[graphql(desc = "The regex pattern of a URL in the Web service. Pass an empty value to remove.")]
        url_pattern: Option<String>,
    ) -> Result<ExternalService> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();
        let normalizer = ctx.data_unchecked::<Arc<Normalizer>>();

        let slug = slug.map(|slug| normalizer.normalize(slug));
        let name = name.map(|name| normalizer.normalize(name));

        let base_url = match &base_url {
            Some(base_url) if base_url.is_empty() => Some(None),
            Some(base_url) => Some(Some(base_url.as_str())),
            None => None,
        };
        let url_pattern = match &url_pattern {
            Some(url_pattern) if url_pattern.is_empty() => Some(None),
            Some(url_pattern) => Some(Some(url_pattern.as_str())),
            None => None,
        };

        let service = external_services_service.update_external_service_by_id(id.into(), slug.as_deref(), name.as_deref(), base_url, url_pattern).await?;
        Ok(service.into())
    }

    /// Deletes an external service.
    async fn delete_external_service(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the ExternalService object.")]
        id: Uuid,
    ) -> Result<DeleteResult> {
        let external_services_service = ctx.data_unchecked::<ExternalServicesService>();

        let result = external_services_service.delete_external_service_by_id(id.into()).await?;
        Ok(result.into())
    }

    /// Creates a medium.
    /// ### Errors
    /// * When any of the sources is not found, it returns a `MEDIUM_SOURCE_NOT_FOUND` error.
    /// * When any of the tags is not found, it returns a `MEDIUM_TAG_NOT_FOUND` error.
    async fn create_medium(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The IDs of Source objects to associate.")]
        source_ids: Option<Vec<Uuid>>,
        #[graphql(desc = "The date at which the medium was created.")]
        created_at: Option<DateTime<FixedOffset>>,
        #[graphql(desc = "The IDs of Tag and TagType objects to associate.")]
        tag_ids: Option<Vec<TagTagTypeInput>>,
    ) -> Result<Medium> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let tags = ctx.look_ahead().field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let sources = ctx.look_ahead().field("sources").exists();

        let source_ids = source_ids.unwrap_or_default().into_iter().map(Into::into);
        let tag_tag_type_ids = tag_ids.unwrap_or_default().into_iter().map(Into::into);

        let created_at = created_at.map(Into::into);

        let medium = media_service.create_medium(source_ids, created_at, tag_tag_type_ids, tag_depth, sources).await?;
        let medium = medium.try_into()?;
        Ok(medium)
    }

    /// Creates a replica either from an `originalUrl` or `upload`. By default, the replica will be processed asynchronously,
    /// hence some fields being unavailable in the response. Watch for the status updates of the medium by Subscription.
    /// ### Errors
    /// * When the medium is not found, it returns a `MEDIUM_NOT_FOUND` error.
    /// * When any replica with the same original URL already exists, it returns a `REPLICA_ORIGINAL_URL_DUPLICATE` error.
    /// * When the object with the same name already exists and `overwrite` is disabled, it returns an `OBJECT_ALREADY_EXISTS` error.
    /// * When the object could not be retrieved from the original URL, it returns an `OBJECT_GET_FAILED` error.
    /// * When the object could not be created, it returns an `OBJECT_PUT_FAILED` error.
    /// * When the original URL or the name of the upload is invalid, it returns an `OBJECT_URL_INVALID` error.
    /// * When the original URL is unsupported, it returns an `OBJECT_URL_UNSUPPORTED` error.
    async fn create_replica(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Medium object to which the replica is appended.")]
        medium_id: Uuid,
        #[graphql(desc = "The original URL of the replica. Mutually exclusive with `upload`.")]
        original_url: Option<String>,
        #[graphql(desc = "The upload of the replica. Mutually exclusive with `originalUrl`.")]
        upload: Option<ReplicaInput>,
        #[graphql(default = false, desc = "Whether to process replica synchronously.")]
        sync: bool,
    ) -> Result<Replica> {
        let media_service = ctx.data_unchecked::<MediaService>();
        let tracker = ctx.data_unchecked::<TaskTracker>();

        let medium_source = create_medium_source(ctx, original_url, upload).await?;
        let (replica, task) = media_service.create_replica(medium_id.into(), medium_source).await?;

        let replica = if sync {
            task.await?
        } else {
            tracker.spawn(task);
            replica
        };

        Ok(replica.into())
    }

    /// Creates a source.
    /// ### Errors
    /// * When the external service is not found, it returns an `EXTERNAL_SERVICE_NOT_FOUND` error.
    /// * When any source with the same metadata already exists, it returns a `SOURCE_METADATA_DUPLICATE` error.
    /// * When the metadata is invalid, it returns a `SOURCE_METADATA_INVALID` error.
    /// * When the metadata does not match with the external service, it returns a `SOURCE_METADATA_NOT_MATCH` error.
    async fn create_source(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the ExternalService object.")]
        external_service_id: Uuid,
        #[graphql(desc = "The metadata from the external service.")]
        external_metadata: ExternalMetadata,
    ) -> Result<Source> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let external_service_id = external_service_id.into();
        let external_metadata = external_metadata.try_into()?;

        let source = media_service.create_source(external_service_id, external_metadata).await?;
        let source = source.try_into()?;
        Ok(source)
    }

    /// Updates a medium. The replicas must match with the current when specifying `replicaOrders`.
    /// ### Errors
    /// * When the medium is not found, it returns a `MEDIUM_NOT_FOUND` error.
    /// * When any of the sources is not found, it returns a `MEDIUM_SOURCE_NOT_FOUND` error.
    /// * When any of the tags is not found, it returns a `MEDIUM_TAG_NOT_FOUND` error.
    /// * When the replicas do not match with the current, it returns a `MEDIUM_REPLICAS_NOT_MATCH` error.
    async fn update_medium(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Medium object.")]
        id: Uuid,
        #[graphql(desc = "The IDs of Source objects to associate.")]
        add_source_ids: Option<Vec<Uuid>>,
        #[graphql(desc = "The IDs of Source objects to dissociate.")]
        remove_source_ids: Option<Vec<Uuid>>,
        #[graphql(desc = "The IDs of Tag and TagType objects to associate.")]
        add_tag_ids: Option<Vec<TagTagTypeInput>>,
        #[graphql(desc = "The IDs of Tag and TagType objects to dissociate.")]
        remove_tag_ids: Option<Vec<TagTagTypeInput>>,
        #[graphql(desc = "The IDs of Replica objects in the order they appear.")]
        replica_orders: Option<Vec<Uuid>>,
        #[graphql(desc = "The date at which the medium was created.")]
        created_at: Option<DateTime<FixedOffset>>,
    ) -> Result<Medium> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let tags = ctx.look_ahead().field("tags").field("tag");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let replicas = ctx.look_ahead().field("replicas").exists();
        let sources = ctx.look_ahead().field("sources").exists();

        let add_source_ids = add_source_ids.unwrap_or_default().into_iter().map(Into::into);
        let remove_source_ids = remove_source_ids.unwrap_or_default().into_iter().map(Into::into);

        let add_tag_tag_type_ids = add_tag_ids.unwrap_or_default().into_iter().map(Into::into);
        let remove_tag_tag_type_ids = remove_tag_ids.unwrap_or_default().into_iter().map(Into::into);

        let replica_orders = replica_orders.unwrap_or_default().into_iter().map(Into::into);

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
        let medium = medium.try_into()?;
        Ok(medium)
    }

    /// Updates a replica either from an `originalUrl` or `upload`. By default, the replica will be processed asynchronously,
    /// hence some fields being unavailable in the response. Watch for the status updates of the medium by Subscription.
    /// ### Errors
    /// * When the medium is not found, it returns a `MEDIUM_NOT_FOUND` error.
    /// * When any replica with the same original URL already exists, it returns a `REPLICA_ORIGINAL_URL_DUPLICATE` error.
    /// * When the object with the same name already exists and `overwrite` is disabled, it returns an `OBJECT_ALREADY_EXISTS` error.
    /// * When the object could not be retrieved from the original URL, it returns an `OBJECT_GET_FAILED` error.
    /// * When the object could not be created, it returns an `OBJECT_PUT_FAILED` error.
    /// * When the original URL or the name of the upload is invalid, it returns an `OBJECT_URL_INVALID` error.
    /// * When the original URL is unsupported, it returns an `OBJECT_URL_UNSUPPORTED` error.
    async fn update_replica(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Replica object.")]
        id: Uuid,
        #[graphql(desc = "The original URL of the replica. Mutually exclusive with `upload`.")]
        original_url: Option<String>,
        #[graphql(desc = "The upload of the replica. Mutually exclusive with `originalUrl`.")]
        upload: Option<ReplicaInput>,
        #[graphql(default = false, desc = "Whether to process replica synchronously.")]
        sync: bool,
    ) -> Result<Replica> {
        let media_service = ctx.data_unchecked::<MediaService>();
        let tracker = ctx.data_unchecked::<TaskTracker>();

        let medium_source = create_medium_source(ctx, original_url, upload).await?;
        let (replica, task) = media_service.update_replica_by_id(id.into(), medium_source).await?;

        let replica = if sync {
            task.await?
        } else {
            tracker.spawn(task);
            replica
        };

        Ok(replica.into())
    }

    /// Creates a source.
    /// ### Errors
    /// * When the source is not found, it returns a `SOURCE_NOT_FOUND` error.
    /// * When the external service is not found, it returns an `EXTERNAL_SERVICE_NOT_FOUND` error.
    /// * When any source with the same metadata already exists, it returns a `SOURCE_METADATA_DUPLICATE` error.
    /// * When the metadata is invalid, it returns a `SOURCE_METADATA_INVALID` error.
    /// * When the metadata does not match with the external service, it returns a `SOURCE_METADATA_NOT_MATCH` error.
    async fn update_source(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Source object.")]
        id: Uuid,
        #[graphql(desc = "The ID of the ExternalService object.")]
        external_service_id: Option<Uuid>,
        #[graphql(desc = "The metadata from the external service.")]
        external_metadata: Option<ExternalMetadata>,
    ) -> Result<Source> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let external_service_id = external_service_id.map(Into::into);
        let external_metadata = external_metadata.map(TryInto::try_into).transpose()?;

        let source = media_service.update_source_by_id(id.into(), external_service_id, external_metadata).await?;
        let source = source.try_into()?;
        Ok(source)
    }

    /// Deletes a medium.
    /// ### Errors
    /// * When the objects in the storage could not be deleted, it returns an `OBJECT_DELETE_FAILED` error.
    async fn delete_medium(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Medium object.")]
        id: Uuid,
        #[graphql(desc = "Whether to delete the associated objects in the storage.")]
        delete_objects: Option<bool>,
    ) -> Result<DeleteResult> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let delete_objects = delete_objects.unwrap_or_default();

        let result = media_service.delete_medium_by_id(id.into(), delete_objects).await?;
        Ok(result.into())
    }

    /// Deletes a replica.
    /// ### Errors
    /// * When the object in the storage could not be deleted, it returns an `OBJECT_DELETE_FAILED` error.
    async fn delete_replica(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Replica object.")]
        id: Uuid,
        #[graphql(desc = "Whether to delete the associated object in the storage.")]
        delete_object: Option<bool>,
    ) -> Result<DeleteResult> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let delete_object = delete_object.unwrap_or_default();

        let result = media_service.delete_replica_by_id(id.into(), delete_object).await?;
        Ok(result.into())
    }

    /// Deletes a source.
    async fn delete_source(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Source object.")]
        id: Uuid,
    ) -> Result<DeleteResult> {
        let media_service = ctx.data_unchecked::<MediaService>();

        let result = media_service.delete_source_by_id(id.into()).await?;
        Ok(result.into())
    }

    /// Creates a tag.
    /// ### Errors
    /// * When the parent tag is not found, it returns a `TAG_NOT_FOUND` error.
    async fn create_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The name of the tag.")]
        name: String,
        #[graphql(desc = "The kana of the tag.")]
        kana: String,
        #[graphql(desc = "The list of aliases for the tag.")]
        aliases: Option<Vec<String>>,
        #[graphql(desc = "The ID of the parent Tag object.")]
        parent_id: Option<Uuid>,
    ) -> Result<Tag> {
        let tags_service = ctx.data_unchecked::<TagsService>();
        let normalizer = ctx.data_unchecked::<Arc<Normalizer>>();

        let name = normalizer.normalize(name);
        let kana = normalizer.normalize(kana);
        let aliases = aliases.unwrap_or_default().into_iter().map({
            let normalizer = normalizer.clone();
            move |alias| normalizer.normalize(alias)
        });
        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = tags_service.create_tag(&name, &kana, aliases, parent_id.map(Into::into), depth).await?;
        Ok(tag.into())
    }

    /// Creates a tag type.
    /// ### Errors
    /// * When the slug already exists, it returns a `TAG_TYPE_SLUG_DUPLICATE` error.
    async fn create_tag_type(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The short and user-friendly name that uniquely identifies the tag type.")]
        slug: String,
        #[graphql(desc = "The name of the tag type.")]
        name: String,
        #[graphql(desc = "The kana of the tag type.")]
        kana: String,
    ) -> Result<TagType> {
        let tags_service = ctx.data_unchecked::<TagsService>();
        let normalizer = ctx.data_unchecked::<Arc<Normalizer>>();

        let slug = normalizer.normalize(slug);
        let name = normalizer.normalize(name);
        let kana = normalizer.normalize(kana);

        let tag_type = tags_service.create_tag_type(&slug, &name, &kana).await?;
        Ok(tag_type.into())
    }

    /// Updates a tag.
    async fn update_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Tag object.")]
        id: Uuid,
        #[graphql(desc = "The name of the tag.")]
        name: Option<String>,
        #[graphql(desc = "The kana of the tag.")]
        kana: Option<String>,
        #[graphql(desc = "The list of aliases to add to the tag.")]
        add_aliases: Option<Vec<String>>,
        #[graphql(desc = "The list of aliases to remove from the tag.")]
        remove_aliases: Option<Vec<String>>,
    ) -> Result<Tag> {
        let tags_service = ctx.data_unchecked::<TagsService>();
        let normalizer = ctx.data_unchecked::<Arc<Normalizer>>();

        let name = name.map(|name| normalizer.normalize(name));
        let kana = kana.map(|kana| normalizer.normalize(kana));
        let add_aliases = add_aliases.unwrap_or_default().into_iter().map({
            let normalizer = normalizer.clone();
            move |alias| normalizer.normalize(alias)
        });
        let remove_aliases = remove_aliases.unwrap_or_default().into_iter().map({
            let normalizer = normalizer.clone();
            move |alias| normalizer.normalize(alias)
        });

        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = tags_service.update_tag_by_id(id.into(), name, kana, add_aliases, remove_aliases, depth).await?;
        Ok(tag.into())
    }

    /// Updates a tag type.
    /// ### Errors
    /// * When the slug already exists, it returns a `TAG_TYPE_SLUG_DUPLICATE` error.
    async fn update_tag_type(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the TagType object.")]
        id: Uuid,
        #[graphql(desc = "The short and user-friendly name that uniquely identifies the tag type.")]
        slug: Option<String>,
        #[graphql(desc = "The name of the tag type.")]
        name: Option<String>,
        #[graphql(desc = "The kana of the tag type.")]
        kana: Option<String>,
    ) -> Result<TagType> {
        let tags_service = ctx.data_unchecked::<TagsService>();
        let normalizer = ctx.data_unchecked::<Arc<Normalizer>>();

        let slug = slug.map(|slug| normalizer.normalize(slug));
        let name = name.map(|name| normalizer.normalize(name));
        let kana = kana.map(|kana| normalizer.normalize(kana));

        let tag_type = tags_service.update_tag_type_by_id(id.into(), slug.as_deref(), name.as_deref(), kana.as_deref()).await?;
        Ok(tag_type.into())
    }

    /// Attaches a tag to another one.
    /// ### Errors
    /// * When the tag is not found, it returns a `TAG_NOT_FOUND` error.
    /// * When the tag is being attached to its descendant, it returns a `TAG_ATTACHING_TO_DESCENDANT` error.
    /// * When the tag is being attached to itself, it returns a `TAG_ATTACHING_TO_ITSELF` error.
    async fn attach_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Tag object.")]
        id: Uuid,
        #[graphql(desc = "The ID of the new parent Tag object.")]
        parent_id: Uuid,
    ) -> Result<Tag> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = tags_service.attach_tag_by_id(id.into(), parent_id.into(), depth).await?;
        Ok(tag.into())
    }

    /// Detaches a tag from its parent.
    /// ### Errors
    /// * When the tag is not found, it returns a `TAG_NOT_FOUND` error.
    async fn detach_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Tag object.")]
        id: Uuid,
    ) -> Result<Tag> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = tags_service.detach_tag_by_id(id.into(), depth).await?;
        Ok(tag.into())
    }

    /// Deletes a tag.
    /// ### Errors
    /// * When the tag has children and `recursive` is disabled, it returns a `TAG_CHILDREN_EXIST` error.
    async fn delete_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Tag object.")]
        id: Uuid,
        #[graphql(default = false, desc = "Whether to delete all the descendants.")]
        recursive: bool,
    ) -> Result<DeleteResult> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let result = tags_service.delete_tag_by_id(id.into(), recursive).await?;
        Ok(result.into())
    }

    /// Deletes a tag type.
    async fn delete_tag_type(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The ID of the Tag Type object.")]
        id: Uuid,
    ) -> Result<DeleteResult> {
        let tags_service = ctx.data_unchecked::<TagsService>();

        let result = tags_service.delete_tag_type_by_id(id.into()).await?;
        Ok(result.into())
    }
}
