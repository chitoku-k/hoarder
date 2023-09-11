use async_graphql::{Context, Object, SimpleObject};
use chrono::{DateTime, FixedOffset};
use derive_more::Constructor;
use domain::{
    repository,
    service::{
        external_services::ExternalServicesServiceInterface,
        media::MediaServiceInterface,
        tags::TagsServiceInterface,
    },
};
use uuid::Uuid;

use crate::{
    external_services::ExternalService,
    media::Medium,
    replicas::Replica,
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

#[Object]
impl<ExternalServicesService, MediaService, TagsService> Mutation<ExternalServicesService, MediaService, TagsService>
where
    ExternalServicesService: ExternalServicesServiceInterface,
    MediaService: MediaServiceInterface,
    TagsService: TagsServiceInterface,
{
    async fn create_external_service(&self, slug: String, name: String) -> anyhow::Result<ExternalService> {
        let service = self.external_services_service.create_external_service(&slug, &name).await?;
        Ok(service.into())
    }

    async fn update_external_service(&self, id: Uuid, name: Option<String>) -> anyhow::Result<ExternalService> {
        let service = self.external_services_service.update_external_service_by_id(id.into(), name.as_deref()).await?;
        Ok(service.into())
    }

    async fn delete_external_service(&self, id: Uuid) -> anyhow::Result<DeleteResult> {
        let result = self.external_services_service.delete_external_service_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn create_medium(
        &self,
        ctx: &Context<'_>,
        source_ids: Option<Vec<Uuid>>,
        created_at: Option<DateTime<FixedOffset>>,
        tag_ids: Option<Vec<TagTagTypeInput>>,
    ) -> anyhow::Result<Medium> {
        let tags = ctx.look_ahead().field("tags");
        let tag_depth = tags.exists().then(|| get_tag_depth(&tags));
        let sources = ctx.look_ahead().field("sources").exists();

        let source_ids: Map<_, _, _> = source_ids.unwrap_or_default().into_iter().map(Into::into);
        let tag_tag_type_ids: Map<_, _, _> = tag_ids.unwrap_or_default().into_iter().map(Into::into);

        let created_at = created_at.map(Into::into);

        let medium = self.media_service.create_medium(source_ids, created_at, tag_tag_type_ids, tag_depth, sources).await?;
        medium.try_into()
    }

    async fn create_replica(&self, medium_id: Uuid, original_url: String) -> anyhow::Result<Replica> {
        let replica = self.media_service.create_replica(medium_id.into(), &original_url).await?;
        Ok(replica.into())
    }

    async fn create_source(&self, external_service_id: Uuid, external_metadata: ExternalMetadata) -> anyhow::Result<Source> {
        let external_service_id = external_service_id.into();
        let external_metadata = external_metadata.try_into()?;

        let source = self.media_service.create_source(external_service_id, external_metadata).await?;
        source.try_into()
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
    ) -> anyhow::Result<Medium> {
        let tags = ctx.look_ahead().field("tags");
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
        medium.try_into()
    }

    async fn update_replica(&self, id: Uuid, original_url: Option<String>) -> anyhow::Result<Replica> {
        let replica = self.media_service.update_replica_by_id(id.into(), original_url.as_deref()).await?;
        Ok(replica.into())
    }

    async fn update_source(&self, id: Uuid, external_service_id: Option<Uuid>, external_metadata: Option<ExternalMetadata>) -> anyhow::Result<Source> {
        let external_service_id = external_service_id.map(Into::into);
        let external_metadata = external_metadata.map(TryInto::try_into).transpose()?;

        let source = self.media_service.update_source_by_id(id.into(), external_service_id, external_metadata).await?;
        source.try_into()
    }

    async fn delete_medium(&self, id: Uuid) -> anyhow::Result<DeleteResult> {
        let result = self.media_service.delete_medium_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn delete_replica(&self, id: Uuid) -> anyhow::Result<DeleteResult> {
        let result = self.media_service.delete_replica_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn delete_source(&self, id: Uuid) -> anyhow::Result<DeleteResult> {
        let result = self.media_service.delete_source_by_id(id.into()).await?;
        Ok(result.into())
    }

    async fn create_tag(&self, ctx: &Context<'_>, name: String, kana: String, aliases: Option<Vec<String>>, parent_id: Option<Uuid>) -> anyhow::Result<Tag> {
        let depth = get_tag_depth(&ctx.look_ahead());
        let aliases = aliases.unwrap_or_default();

        let tag = self.tags_service.create_tag(&name, &kana, &aliases, parent_id.map(Into::into), depth).await?;
        Ok(tag.into())
    }

    async fn create_tag_type(&self, slug: String, name: String) -> anyhow::Result<TagType> {
        let tag_type = self.tags_service.create_tag_type(&slug, &name).await?;
        Ok(tag_type.into())
    }

    async fn update_tag(&self, ctx: &Context<'_>, id: Uuid, name: Option<String>, kana: Option<String>, add_aliases: Option<Vec<String>>, remove_aliases: Option<Vec<String>>) -> anyhow::Result<Tag> {
        let depth = get_tag_depth(&ctx.look_ahead());
        let add_aliases = add_aliases.unwrap_or_default();
        let remove_aliases = remove_aliases.unwrap_or_default();

        let tag = self.tags_service.update_tag_by_id(id.into(), name, kana, add_aliases, remove_aliases, depth).await?;
        Ok(tag.into())
    }

    async fn update_tag_type(&self, id: Uuid, slug: Option<String>, name: Option<String>) -> anyhow::Result<TagType> {
        let tag_type = self.tags_service.update_tag_type_by_id(id.into(), slug.as_deref(), name.as_deref()).await?;
        Ok(tag_type.into())
    }

    async fn attach_tag(&self, ctx: &Context<'_>, id: Uuid, parent_id: Uuid) -> anyhow::Result<Tag> {
        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = self.tags_service.attach_tag_by_id(id.into(), parent_id.into(), depth).await?;
        Ok(tag.into())
    }

    async fn detach_tag(&self, ctx: &Context<'_>, id: Uuid) -> anyhow::Result<Tag> {
        let depth = get_tag_depth(&ctx.look_ahead());

        let tag = self.tags_service.detach_tag_by_id(id.into(), depth).await?;
        Ok(tag.into())
    }

    async fn delete_tag(
        &self,
        id: Uuid,
        #[graphql(default = false)]
        recursive: bool,
    ) -> anyhow::Result<DeleteResult> {
        let result = self.tags_service.delete_tag_by_id(id.into(), recursive).await?;
        Ok(result.into())
    }

    async fn delete_tag_type(&self, id: Uuid) -> anyhow::Result<DeleteResult> {
        let result = self.tags_service.delete_tag_type_by_id(id.into()).await?;
        Ok(result.into())
    }
}
