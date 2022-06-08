use async_graphql::{InputObject, OneofObject, SimpleObject};
use chrono::NaiveDateTime;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use uuid::Uuid;

use crate::{
    application::graphql::external_services::ExternalService,
    domain::entity::{external_services, sources},
};

#[derive(SimpleObject)]
pub struct Source {
    id: Uuid,
    external_service: ExternalService,
    external_metadata: serde_json::Value,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(OneofObject, Serialize)]
#[graphql(name = "ExternalMetadataInput")]
#[serde(rename_all = "camelCase")]
pub enum ExternalMetadata {
    Fantia(ExternalMetadataId),
    Nijie(ExternalMetadataId),
    Pixiv(ExternalMetadataId),
    PixivFanbox(ExternalMetadataIdCreatorId),
    Seiga(ExternalMetadataId),
    Skeb(ExternalMetadataIdCreatorId),
    Twitter(ExternalMetadataId),
    Website(ExternalMetadataUrl),
    Custom(serde_json::Value),
}

#[serde_as]
#[derive(InputObject, Serialize)]
#[graphql(name = "ExternalMetadataIdInput")]
pub struct ExternalMetadataId {
    #[serde_as(as = "DisplayFromStr")]
    id: u64,
}

#[serde_as]
#[derive(InputObject, Serialize)]
#[graphql(name = "ExternalMetadataIdCreatorIdInput")]
pub struct ExternalMetadataIdCreatorId {
    #[serde_as(as = "DisplayFromStr")]
    id: u64,
    creator_id: String,
}

#[derive(InputObject, Serialize)]
#[graphql(name = "ExternalMetadataUrlInput")]
pub struct ExternalMetadataUrl {
    url: String,
}

impl TryFrom<external_services::ExternalMetadata> for ExternalMetadata {
    type Error = anyhow::Error;

    fn try_from(value: external_services::ExternalMetadata) -> anyhow::Result<Self> {
        use external_services::ExternalMetadata::*;
        match value {
            Fantia { id } => Ok(Self::Fantia(ExternalMetadataId { id })),
            Nijie { id } => Ok(Self::Nijie(ExternalMetadataId { id })),
            Pixiv { id } => Ok(Self::Pixiv(ExternalMetadataId { id })),
            PixivFanbox { id, creator_id } => Ok(Self::PixivFanbox(ExternalMetadataIdCreatorId { id, creator_id })),
            Seiga { id } => Ok(Self::Seiga(ExternalMetadataId { id })),
            Skeb { id, creator_id } => Ok(Self::Skeb(ExternalMetadataIdCreatorId { id, creator_id })),
            Twitter { id } => Ok(Self::Twitter(ExternalMetadataId { id })),
            Website { url } => Ok(Self::Website(ExternalMetadataUrl { url })),
            Custom(v) => Ok(Self::Custom(serde_json::from_str(&v)?)),
        }
    }
}

impl TryFrom<ExternalMetadata> for external_services::ExternalMetadata {
    type Error = anyhow::Error;

    fn try_from(value: ExternalMetadata) -> anyhow::Result<Self> {
        use ExternalMetadata::*;
        match value {
            Fantia(ExternalMetadataId { id }) => Ok(Self::Fantia { id }),
            Nijie(ExternalMetadataId { id }) => Ok(Self::Nijie { id }),
            Pixiv(ExternalMetadataId { id }) => Ok(Self::Pixiv { id }),
            PixivFanbox(ExternalMetadataIdCreatorId { id, creator_id }) => Ok(Self::PixivFanbox { id, creator_id }),
            Seiga(ExternalMetadataId { id }) => Ok(Self::Seiga { id }),
            Skeb(ExternalMetadataIdCreatorId { id, creator_id }) => Ok(Self::Skeb { id, creator_id }),
            Twitter(ExternalMetadataId { id }) => Ok(Self::Twitter { id }),
            Website(ExternalMetadataUrl { url }) => Ok(Self::Website { url }),
            Custom(v) => Ok(Self::Custom(serde_json::to_string(&v)?)),
        }
    }
}

impl TryFrom<sources::Source> for Source {
    type Error = anyhow::Error;

    fn try_from(source: sources::Source) -> anyhow::Result<Self> {
        let external_metadata = ExternalMetadata::try_from(source.external_metadata)?;

        Ok(Self {
            id: *source.id,
            external_service: source.external_service.into(),
            external_metadata: serde_json::to_value(&external_metadata)?,
            created_at: source.created_at,
            updated_at: source.updated_at,
        })
    }
}
