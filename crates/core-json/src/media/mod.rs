// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use num_traits::{FromPrimitive as _, ToPrimitive as _};
use url::Url;

use aoide_core::{media::ContentMetadataFlags, util::url::BaseUrl};

use crate::{audio::AudioContent, prelude::*, util::clock::DateTime};

mod _core {
    pub use aoide_core::media::*;
}

pub use _core::SourcePath;

#[derive(Debug, Serialize_repr, Deserialize_repr, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[repr(u8)]
pub enum SourcePathKind {
    Uri = _core::SourcePathKind::Uri as u8,
    Url = _core::SourcePathKind::Url as u8,
    FileUrl = _core::SourcePathKind::FileUrl as u8,
    VirtualFilePath = _core::SourcePathKind::VirtualFilePath as u8,
}

impl From<_core::SourcePathKind> for SourcePathKind {
    fn from(from: _core::SourcePathKind) -> Self {
        use _core::SourcePathKind::*;
        match from {
            Uri => Self::Uri,
            Url => Self::Url,
            FileUrl => Self::FileUrl,
            VirtualFilePath => Self::VirtualFilePath,
        }
    }
}

impl From<SourcePathKind> for _core::SourcePathKind {
    fn from(from: SourcePathKind) -> Self {
        use SourcePathKind::*;
        match from {
            Uri => Self::Uri,
            Url => Self::Url,
            FileUrl => Self::FileUrl,
            VirtualFilePath => Self::VirtualFilePath,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourcePathConfig {
    pub path_kind: SourcePathKind,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,
}

impl TryFrom<SourcePathConfig> for _core::SourcePathConfig {
    type Error = anyhow::Error;

    fn try_from(from: SourcePathConfig) -> anyhow::Result<Self> {
        let SourcePathConfig {
            path_kind,
            root_url,
        } = from;
        let into = match path_kind {
            SourcePathKind::Uri => Self::Uri,
            SourcePathKind::Url => Self::Url,
            SourcePathKind::FileUrl => Self::FileUrl,
            SourcePathKind::VirtualFilePath => {
                if let Some(root_url) = root_url {
                    let root_url = match BaseUrl::try_from(root_url) {
                        Ok(root_url) => root_url,
                        Err(err) => {
                            anyhow::bail!("Invalid root URL: {}", err);
                        }
                    };
                    Self::VirtualFilePath { root_url }
                } else {
                    anyhow::bail!("Missing root URL");
                }
            }
        };
        Ok(into)
    }
}

impl From<_core::SourcePathConfig> for SourcePathConfig {
    fn from(from: _core::SourcePathConfig) -> Self {
        let (path_kind, root_url) = from.into();
        Self {
            path_kind: path_kind.into(),
            root_url: root_url.map(Into::into),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Content
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Base64(String);

impl Base64 {
    pub fn encode(bytes: impl AsRef<[u8]>) -> Self {
        let encoded = base64::encode_config(bytes.as_ref(), base64::URL_SAFE_NO_PAD);
        Self(encoded)
    }

    pub fn try_decode(&self) -> Result<Vec<u8>, base64::DecodeError> {
        let Self(encoded) = self;
        Self::try_decode_impl(encoded)
    }

    fn try_decode_impl(encoded: impl AsRef<str>) -> Result<Vec<u8>, base64::DecodeError> {
        base64::decode_config(encoded.as_ref(), base64::URL_SAFE_NO_PAD)
    }

    pub fn from_encoded(encoded: impl Into<String>) -> Self {
        let encoded = encoded.into();
        debug_assert!(Self::try_decode_impl(&encoded).is_ok());
        Self(encoded)
    }
}

impl AsRef<str> for Base64 {
    fn as_ref(&self) -> &str {
        let Self(encoded) = self;
        encoded
    }
}

impl<T> From<T> for Base64
where
    T: AsRef<[u8]>,
{
    fn from(from: T) -> Self {
        Self::encode(from)
    }
}

impl TryFrom<&Base64> for Vec<u8> {
    type Error = base64::DecodeError;

    fn try_from(from: &Base64) -> Result<Self, Self::Error> {
        from.try_decode()
    }
}

// TODO: Use a more efficient serialization for fixed-length data
// https://github.com/signalapp/SecureValueRecovery/blob/master/service/kbupd_util/src/base64.rs
pub type Digest = Base64;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct DigestRef<'a>(&'a str);

impl<'a> AsRef<str> for DigestRef<'a> {
    fn as_ref(&self) -> &str {
        let DigestRef(encoded) = self;
        encoded
    }
}

impl<'a> TryFrom<DigestRef<'a>> for Vec<u8> {
    type Error = base64::DecodeError;

    fn try_from(from: DigestRef<'a>) -> Result<Self, Self::Error> {
        let DigestRef(encoded) = from;
        Digest::try_decode_impl(encoded)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub enum Content {
    Audio(AudioContent),
}

impl From<Content> for _core::Content {
    fn from(from: Content) -> Self {
        use _core::Content::*;
        match from {
            Content::Audio(audio_content) => Audio(audio_content.into()),
        }
    }
}

impl From<_core::Content> for Content {
    fn from(from: _core::Content) -> Self {
        use _core::Content::*;
        match from {
            Audio(audio_content) => Content::Audio(audio_content.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Artwork
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(deny_unknown_fields)]
pub struct ImageSize(u16, u16);

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkImage {
    media_type: String,

    apic_type: u8,

    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<ImageSize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<Digest>,

    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<Base64>,
}

impl From<_core::ArtworkImage> for ArtworkImage {
    fn from(from: _core::ArtworkImage) -> Self {
        let _core::ArtworkImage {
            media_type,
            apic_type,
            size,
            digest,
            thumbnail,
        } = from;
        let size = size.map(|size| {
            let _core::ImageSize { width, height } = size;
            ImageSize(width, height)
        });
        Self {
            media_type: media_type.to_string(),
            apic_type: apic_type.to_u8().expect("u8"),
            size,
            digest: digest.as_ref().map(Into::into),
            thumbnail: thumbnail.as_ref().map(Into::into),
        }
    }
}

impl TryFrom<ArtworkImage> for _core::ArtworkImage {
    type Error = anyhow::Error;

    fn try_from(from: ArtworkImage) -> anyhow::Result<Self> {
        let ArtworkImage {
            media_type,
            apic_type,
            size,
            digest,
            thumbnail,
        } = from;
        let media_type = media_type.parse()?;
        let apic_type = _core::ApicType::from_u8(apic_type)
            .ok_or_else(|| anyhow::anyhow!("Invalid APIC type: {}", apic_type))?;
        let size = size.map(|size| {
            let ImageSize(width, height) = size;
            _core::ImageSize { width, height }
        });
        let digest_data = digest.as_ref().map(Vec::try_from).transpose()?;
        let digest = digest_data
            .map(TryFrom::try_from)
            .transpose()
            .map_err(|_| anyhow::anyhow!("Failed to deserialize artwork digest"))?;
        let thumbnail_data = thumbnail.as_ref().map(Vec::try_from).transpose()?;
        let thumbnail = thumbnail_data
            .map(TryFrom::try_from)
            .transpose()
            .map_err(|_| anyhow::anyhow!("Failed to deserialize artwork thumbnail"))?;
        let into = Self {
            media_type,
            apic_type,
            size,
            digest,
            thumbnail,
        };
        Ok(into)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "kebab-case")]
pub enum ArtworkSource {
    Missing,
    Unsupported,
    Irregular,
    Embedded,
    Linked,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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
        use _core::Artwork::*;
        match from {
            Missing => Self {
                source: ArtworkSource::Missing,
                uri: None,
                image: None,
            },
            Unsupported => Self {
                source: ArtworkSource::Unsupported,
                uri: None,
                image: None,
            },
            Irregular => Self {
                source: ArtworkSource::Irregular,
                uri: None,
                image: None,
            },
            Embedded(embedded) => Self {
                source: ArtworkSource::Embedded,
                uri: None,
                image: Some(embedded.image.into()),
            },
            Linked(linked) => Self {
                source: ArtworkSource::Linked,
                uri: Some(linked.uri),
                image: Some(linked.image.into()),
            },
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[repr(u8)]
pub enum AdvisoryRating {
    Unrated = _core::AdvisoryRating::Unrated as u8,
    Explicit = _core::AdvisoryRating::Explicit as u8,
    Clean = _core::AdvisoryRating::Clean as u8,
}

impl From<_core::AdvisoryRating> for AdvisoryRating {
    fn from(from: _core::AdvisoryRating) -> Self {
        use _core::AdvisoryRating::*;
        match from {
            Unrated => Self::Unrated,
            Explicit => Self::Explicit,
            Clean => Self::Clean,
        }
    }
}

impl From<AdvisoryRating> for _core::AdvisoryRating {
    fn from(from: AdvisoryRating) -> Self {
        use AdvisoryRating::*;
        match from {
            Unrated => Self::Unrated,
            Explicit => Self::Explicit,
            Clean => Self::Clean,
        }
    }
}

fn is_default_content_metadata_flags(flags: &u8) -> bool {
    *flags == u8::default()
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    collected_at: DateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    synchronized_at: Option<DateTime>,

    path: String,

    content_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    advisory_rating: Option<AdvisoryRating>,

    #[serde(skip_serializing_if = "Option::is_none")]
    content_digest: Option<Digest>,

    #[serde(skip_serializing_if = "is_default_content_metadata_flags", default)]
    content_metadata_flags: u8,

    #[serde(flatten)]
    content: Content,

    #[serde(skip_serializing_if = "Option::is_none")]
    artwork: Option<Artwork>,
}

impl From<_core::Source> for Source {
    fn from(from: _core::Source) -> Self {
        let _core::Source {
            collected_at,
            synchronized_at,
            path,
            content_type,
            content_digest,
            advisory_rating,
            content_metadata_flags,
            content,
            artwork,
        } = from;
        Self {
            collected_at: collected_at.into(),
            synchronized_at: synchronized_at.map(Into::into),
            path: path.into(),
            content_type: content_type.to_string(),
            advisory_rating: advisory_rating.map(Into::into),
            content_digest: content_digest.map(Into::into),
            content_metadata_flags: content_metadata_flags.bits(),
            content: content.into(),
            artwork: artwork.map(Into::into),
        }
    }
}

impl TryFrom<Source> for _core::Source {
    type Error = anyhow::Error;

    fn try_from(from: Source) -> anyhow::Result<Self> {
        let Source {
            collected_at,
            synchronized_at,
            path,
            content_type,
            content_digest,
            advisory_rating,
            content_metadata_flags,
            content,
            artwork,
        } = from;
        let content_type = content_type.parse()?;
        let content_digest = content_digest.as_ref().map(TryFrom::try_from).transpose()?;
        let artwork = artwork.map(TryFrom::try_from).transpose()?;
        let into = Self {
            collected_at: collected_at.into(),
            synchronized_at: synchronized_at.map(Into::into),
            path: path.into(),
            content_type,
            advisory_rating: advisory_rating.map(Into::into),
            content_digest,
            content_metadata_flags: ContentMetadataFlags::from_bits_truncate(
                content_metadata_flags,
            ),
            content: content.into(),
            artwork,
        };
        Ok(into)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
