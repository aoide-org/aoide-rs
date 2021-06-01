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

mod _core {
    pub use aoide_core::media::*;
}

use aoide_core::{
    media::{ContentMetadataFlags, Thumbnail4x4Rgb8},
    util::IsDefault,
};

use crate::{audio::AudioContent, prelude::*, util::clock::DateTime};

use std::convert::TryFrom;

pub use _core::SourcePath;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr, JsonSchema)]
#[repr(u8)]
pub enum SourcePathKind {
    Uri = 0,
    Url = 1,
    FileUrl = 2,
    VirtualFilePath = 3,
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

///////////////////////////////////////////////////////////////////////
// Content
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DigestRef<'a>(&'a str);

impl<'a> AsRef<str> for DigestRef<'a> {
    fn as_ref(&self) -> &str {
        let DigestRef(ref encoded) = self;
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSize(u16, u16);

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Artwork {
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<Digest>,

    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<ImageSize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<Base64>,
}

impl From<_core::Artwork> for Artwork {
    fn from(from: _core::Artwork) -> Self {
        let _core::Artwork {
            uri,
            media_type,
            digest,
            size,
            thumbnail,
        } = from;
        let size = size.map(|size| {
            let _core::ImageSize { width, height } = size;
            ImageSize(width, height)
        });
        Self {
            uri,
            media_type,
            digest: digest.as_ref().map(Into::into),
            size,
            thumbnail: thumbnail.as_ref().map(Into::into),
        }
    }
}

impl From<Artwork> for _core::Artwork {
    fn from(from: Artwork) -> Self {
        let Artwork {
            uri,
            media_type,
            digest,
            size,
            thumbnail,
        } = from;
        let size = size.map(|size| {
            let ImageSize(width, height) = size;
            _core::ImageSize { width, height }
        });
        Self {
            uri,
            media_type,
            digest: digest
                .as_ref()
                .map(Vec::try_from)
                .and_then(Result::ok)
                .map(_core::Digest::try_from)
                .and_then(Result::ok),
            size,
            thumbnail: thumbnail
                .as_ref()
                .map(Vec::try_from)
                .and_then(Result::ok)
                .map(Thumbnail4x4Rgb8::try_from)
                .and_then(Result::ok),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    collected_at: DateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    synchronized_at: Option<DateTime>,

    path: String,

    content_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    content_digest: Option<Digest>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    content_metadata_flags: u8,

    #[serde(flatten)]
    content: Content,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    artwork: Artwork,
}

impl From<_core::Source> for Source {
    fn from(from: _core::Source) -> Self {
        let _core::Source {
            collected_at,
            synchronized_at,
            path,
            content_type,
            content_digest,
            content_metadata_flags,
            content,
            artwork,
        } = from;
        Self {
            collected_at: collected_at.into(),
            synchronized_at: synchronized_at.map(Into::into),
            path: path.into(),
            content_type,
            content_digest: content_digest.as_ref().map(Into::into),
            content_metadata_flags: content_metadata_flags.bits(),
            content: content.into(),
            artwork: artwork.into(),
        }
    }
}

impl From<Source> for _core::Source {
    fn from(from: Source) -> Self {
        let Source {
            collected_at,
            synchronized_at,
            path,
            content_type,
            content_digest,
            content_metadata_flags,
            content,
            artwork,
        } = from;
        Self {
            collected_at: collected_at.into(),
            synchronized_at: synchronized_at.map(Into::into),
            path: path.into(),
            content_type,
            content_digest: content_digest
                .as_ref()
                .map(TryFrom::try_from)
                .and_then(Result::ok),
            content_metadata_flags: ContentMetadataFlags::from_bits_truncate(
                content_metadata_flags,
            ),
            content: content.into(),
            artwork: artwork.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
