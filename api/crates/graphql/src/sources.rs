use async_graphql::{InputObject, OneofObject, SimpleObject};
use chrono::{DateTime, Utc};
use domain::entity::{external_services, sources};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    error::ErrorKind,
    external_services::ExternalService,
};

/// A source represents metadata that uniquely identifies the original location of a medium.
#[derive(SimpleObject)]
pub(crate) struct Source {
    /// The ID of the Source object.
    id: Uuid,
    /// The external service of the source.
    external_service: ExternalService,
    /// The metadata from the external service.
    external_metadata: serde_json::Value,
    /// The URL of the source.
    url: Option<String>,
    /// The date at which the source was created.
    created_at: DateTime<Utc>,
    /// The date at which the source was updated.
    updated_at: DateTime<Utc>,
}

/// An external metadata represents the attributes from the external service.
#[derive(Debug, Eq, OneofObject, PartialEq, Serialize)]
#[graphql(name = "ExternalMetadataInput")]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExternalMetadata {
    /// The metadata from Bluesky.
    Bluesky(ExternalMetadataIdCreatorId),
    /// The metadata from Fantia.
    Fantia(ExternalMetadataId),
    /// The metadata from Mastodon.
    Mastodon(ExternalMetadataIdCreatorId),
    /// The metadata from Misskey.
    Misskey(ExternalMetadataId),
    /// The metadata from ニジエ.
    Nijie(ExternalMetadataId),
    /// The metadata from pixiv.
    Pixiv(ExternalMetadataId),
    /// The metadata from pixivFANBOX.
    #[graphql(name = "pixiv_fanbox")]
    PixivFanbox(ExternalMetadataIdCreatorId),
    /// The metadata from Pleroma.
    Pleroma(ExternalMetadataId),
    /// The metadata from ニコニコ静画.
    Seiga(ExternalMetadataId),
    /// The metadata from Skeb.
    Skeb(ExternalMetadataIdCreatorId),
    /// The metadata from Threads.
    Threads(ExternalMetadataIdOptionalCreatorId),
    /// The URL of any arbitrary website.
    Website(ExternalMetadataUrl),
    /// The metadata from X.
    X(ExternalMetadataIdOptionalCreatorId),
    /// The metadata from Xfolio.
    Xfolio(ExternalMetadataIdCreatorId),
    /// The metadata with a custom value.
    Custom(serde_json::Value),
}

/// An external metadata like represents a partial metadata.
#[derive(Debug, Eq, OneofObject, PartialEq, Serialize)]
#[graphql(name = "ExternalMetadataLikeInput")]
pub(crate) enum ExternalMetadataLike {
    /// The ID of a medium in the external service.
    Id(String),
    /// The URL of a medium in the external service.
    Url(String),
}

/// An external metadata ID represents the ID of a medium in the external service.
#[derive(Debug, Eq, InputObject, PartialEq, Serialize)]
#[graphql(name = "ExternalMetadataIdInput")]
pub(crate) struct ExternalMetadataId {
    /// The ID of a medium in the external service.
    pub id: String,
}

/// An external metadata ID creator ID represents the ID of a medium and creator in the external service.
#[derive(Debug, Eq, InputObject, PartialEq, Serialize)]
#[graphql(name = "ExternalMetadataIdCreatorIdInput")]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExternalMetadataIdCreatorId {
    /// The ID of a medium in the external service.
    pub id: String,
    /// The ID of a creator in the external service.
    pub creator_id: String,
}

/// An external metadata ID optional creator ID represents the ID of a medium and optional creator in the external service.
#[derive(Debug, Eq, InputObject, PartialEq, Serialize)]
#[graphql(name = "ExternalMetadataIdOptionalCreatorIdInput")]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExternalMetadataIdOptionalCreatorId {
    /// The ID of a medium in the external service.
    pub id: String,
    /// The ID of an optional creator in the external service.
    pub creator_id: Option<String>,
}

/// An external metadata URL represents the URL of a medium in the external service.
#[derive(Debug, Eq, InputObject, PartialEq, Serialize)]
#[graphql(name = "ExternalMetadataUrlInput")]
pub(crate) struct ExternalMetadataUrl {
    /// The URL of a medium in the external service.
    pub url: String,
}

impl TryFrom<external_services::ExternalMetadata> for ExternalMetadata {
    type Error = ErrorKind;

    fn try_from(value: external_services::ExternalMetadata) -> Result<Self, Self::Error> {
        use external_services::ExternalMetadata::*;
        match value {
            Bluesky { id, creator_id } => Ok(Self::Bluesky(ExternalMetadataIdCreatorId { id, creator_id })),
            Fantia { id } => Ok(Self::Fantia(ExternalMetadataId { id: id.to_string() })),
            Mastodon { id, creator_id } => Ok(Self::Mastodon(ExternalMetadataIdCreatorId { id: id.to_string(), creator_id })),
            Misskey { id } => Ok(Self::Misskey(ExternalMetadataId { id })),
            Nijie { id } => Ok(Self::Nijie(ExternalMetadataId { id: id.to_string() })),
            Pixiv { id } => Ok(Self::Pixiv(ExternalMetadataId { id: id.to_string() })),
            PixivFanbox { id, creator_id } => Ok(Self::PixivFanbox(ExternalMetadataIdCreatorId { id: id.to_string(), creator_id })),
            Pleroma { id } => Ok(Self::Pleroma(ExternalMetadataId { id })),
            Seiga { id } => Ok(Self::Seiga(ExternalMetadataId { id: id.to_string() })),
            Skeb { id, creator_id } => Ok(Self::Skeb(ExternalMetadataIdCreatorId { id: id.to_string(), creator_id })),
            Threads { id, creator_id } => Ok(Self::Threads(ExternalMetadataIdOptionalCreatorId { id, creator_id })),
            Website { url } => Ok(Self::Website(ExternalMetadataUrl { url })),
            X { id, creator_id } => Ok(Self::X(ExternalMetadataIdOptionalCreatorId { id: id.to_string(), creator_id })),
            Xfolio { id, creator_id } => Ok(Self::Xfolio(ExternalMetadataIdCreatorId { id: id.to_string(), creator_id })),
            Custom(v) => Ok(Self::Custom(serde_json::from_str(&v).map_err(|_| ErrorKind::SourceMetadataInvalid)?)),
        }
    }
}

impl TryFrom<ExternalMetadata> for external_services::ExternalMetadata {
    type Error = ErrorKind;

    fn try_from(value: ExternalMetadata) -> Result<Self, Self::Error> {
        use ExternalMetadata::*;
        match value {
            Bluesky(ExternalMetadataIdCreatorId { id, creator_id }) => Ok(Self::Bluesky { id, creator_id }),
            Fantia(ExternalMetadataId { id }) => Ok(Self::Fantia { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)? }),
            Mastodon(ExternalMetadataIdCreatorId { id, creator_id }) => Ok(Self::Mastodon { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)?, creator_id }),
            Misskey(ExternalMetadataId { id }) => Ok(Self::Misskey { id }),
            Nijie(ExternalMetadataId { id }) => Ok(Self::Nijie { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)? }),
            Pixiv(ExternalMetadataId { id }) => Ok(Self::Pixiv { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)? }),
            PixivFanbox(ExternalMetadataIdCreatorId { id, creator_id }) => Ok(Self::PixivFanbox { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)?, creator_id }),
            Pleroma(ExternalMetadataId { id }) => Ok(Self::Pleroma { id }),
            Seiga(ExternalMetadataId { id }) => Ok(Self::Seiga { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)? }),
            Skeb(ExternalMetadataIdCreatorId { id, creator_id }) => Ok(Self::Skeb { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)?, creator_id }),
            Threads(ExternalMetadataIdOptionalCreatorId { id, creator_id }) => Ok(Self::Threads { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)?, creator_id }),
            Website(ExternalMetadataUrl { url }) => Ok(Self::Website { url }),
            X(ExternalMetadataIdOptionalCreatorId { id, creator_id }) => Ok(Self::X { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)?, creator_id }),
            Xfolio(ExternalMetadataIdCreatorId { id, creator_id }) => Ok(Self::Xfolio { id: id.parse().map_err(|_| ErrorKind::SourceMetadataInvalid)?, creator_id }),
            Custom(v) => Ok(Self::Custom(v.to_string())),
        }
    }
}

impl TryFrom<sources::Source> for Source {
    type Error = ErrorKind;

    fn try_from(source: sources::Source) -> Result<Self, Self::Error> {
        let url = source.url();
        let external_metadata = ExternalMetadata::try_from(source.external_metadata)?;

        Ok(Self {
            id: *source.id,
            external_service: source.external_service.into(),
            external_metadata: serde_json::to_value(external_metadata).map_err(|_| ErrorKind::SourceMetadataInvalid)?,
            url,
            created_at: source.created_at,
            updated_at: source.updated_at,
        })
    }
}
