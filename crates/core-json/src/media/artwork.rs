// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Base64, Digest};
use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::media::artwork::*;
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ImageSize(u16, u16);

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkImage {
    apic_type: u8,

    media_type: String,

    data_size: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<Digest>,

    #[serde(skip_serializing_if = "Option::is_none")]
    image_size: Option<ImageSize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<RgbColor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<Base64>,
}

impl From<_core::ArtworkImage> for ArtworkImage {
    fn from(from: _core::ArtworkImage) -> Self {
        let _core::ArtworkImage {
            apic_type,
            media_type,
            data_size,
            digest,
            image_size,
            color,
            thumbnail,
        } = from;
        let media_type = media_type.to_string();
        let digest = digest.as_ref().map(Into::into);
        let image_size = image_size.map(|size| {
            let _core::ImageSize { width, height } = size;
            ImageSize(width, height)
        });
        let color = color.map(Into::into);
        let thumbnail = thumbnail.as_ref().map(Into::into);
        Self {
            apic_type: apic_type as _,
            media_type,
            data_size,
            digest,
            image_size,
            color,
            thumbnail,
        }
    }
}

impl TryFrom<ArtworkImage> for _core::ArtworkImage {
    type Error = anyhow::Error;

    fn try_from(from: ArtworkImage) -> anyhow::Result<Self> {
        let ArtworkImage {
            apic_type,
            media_type,
            data_size,
            digest,
            image_size,
            color,
            thumbnail,
        } = from;
        let apic_type = _core::ApicType::from_repr(apic_type)
            .ok_or_else(|| anyhow::anyhow!("invalid APIC type: {apic_type}"))?;
        let media_type = media_type.parse()?;
        let digest_data = digest.as_ref().map(Vec::try_from).transpose()?;
        let digest = digest_data
            .map(TryInto::try_into)
            .transpose()
            .map_err(|_| anyhow::anyhow!("failed to deserialize artwork digest"))?;
        let image_size = image_size.map(|size| {
            let ImageSize(width, height) = size;
            _core::ImageSize { width, height }
        });
        let color = color.map(Into::into);
        let thumbnail_data = thumbnail.as_ref().map(Vec::try_from).transpose()?;
        let thumbnail = thumbnail_data
            .map(TryInto::try_into)
            .transpose()
            .map_err(|_| anyhow::anyhow!("failed to deserialize artwork thumbnail"))?;
        let into = Self {
            apic_type,
            media_type,
            data_size,
            digest,
            image_size,
            color,
            thumbnail,
        };
        Ok(into)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ArtworkSource {
    Missing,
    Unsupported,
    Irregular,
    Embedded,
    Linked,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Artwork {
    source: ArtworkSource,

    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<ArtworkImage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    uri: Option<String>,
}

impl TryFrom<Artwork> for _core::Artwork {
    type Error = anyhow::Error;

    fn try_from(from: Artwork) -> anyhow::Result<Self> {
        let Artwork { source, uri, image } = from;
        match source {
            ArtworkSource::Missing => {
                debug_assert!(uri.is_none());
                debug_assert!(image.is_none());
                Ok(_core::Artwork::Missing)
            }
            ArtworkSource::Unsupported => {
                debug_assert!(uri.is_none());
                debug_assert!(image.is_none());
                Ok(_core::Artwork::Unsupported)
            }
            ArtworkSource::Irregular => {
                debug_assert!(uri.is_none());
                debug_assert!(image.is_none());
                Ok(_core::Artwork::Irregular)
            }
            ArtworkSource::Embedded => {
                debug_assert!(uri.is_none());
                if let Some(image) = image {
                    let embedded = _core::EmbeddedArtwork {
                        image: image.try_into()?,
                    };
                    Ok(_core::Artwork::Embedded(embedded))
                } else {
                    anyhow::bail!("missing image for embedded artwork");
                }
            }
            ArtworkSource::Linked => {
                if let (Some(uri), Some(image)) = (uri, image) {
                    let linked = _core::LinkedArtwork {
                        uri,
                        image: image.try_into()?,
                    };
                    Ok(_core::Artwork::Linked(linked))
                } else {
                    anyhow::bail!("missing URI or image for linked artwork");
                }
            }
        }
    }
}

impl From<_core::Artwork> for Artwork {
    fn from(from: _core::Artwork) -> Self {
        use _core::Artwork as From;
        match from {
            From::Missing => Self {
                source: ArtworkSource::Missing,
                uri: None,
                image: None,
            },
            From::Unsupported => Self {
                source: ArtworkSource::Unsupported,
                uri: None,
                image: None,
            },
            From::Irregular => Self {
                source: ArtworkSource::Irregular,
                uri: None,
                image: None,
            },
            From::Embedded(embedded) => Self {
                source: ArtworkSource::Embedded,
                uri: None,
                image: Some(embedded.image.into()),
            },
            From::Linked(linked) => Self {
                source: ArtworkSource::Linked,
                uri: Some(linked.uri),
                image: Some(linked.image.into()),
            },
        }
    }
}
