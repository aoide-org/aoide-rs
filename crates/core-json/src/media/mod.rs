// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::media::content::ContentMetadataFlags;
use data_encoding::{BASE64URL_NOPAD, Encoding};

use self::{
    artwork::Artwork,
    content::{ContentLink, ContentMetadata},
};
use crate::{prelude::*, util::clock::DateTime};

pub mod artwork;
pub mod content;

mod _core {
    pub(super) use aoide_core::media::*;
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Base64(String);

const BASE64: Encoding = BASE64URL_NOPAD;

impl Base64 {
    pub fn encode(bytes: impl AsRef<[u8]>) -> Self {
        let encoded = BASE64.encode(bytes.as_ref());
        Self(encoded)
    }

    pub fn try_decode(&self) -> Result<Vec<u8>, data_encoding::DecodeError> {
        let Self(encoded) = self;
        Self::try_decode_impl(encoded)
    }

    fn try_decode_impl(encoded: impl AsRef<str>) -> Result<Vec<u8>, data_encoding::DecodeError> {
        BASE64.decode(encoded.as_ref().as_bytes())
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
    type Error = data_encoding::DecodeError;

    fn try_from(from: &Base64) -> Result<Self, Self::Error> {
        from.try_decode()
    }
}

// TODO: Use a more efficient serialization for fixed-length data
// https://github.com/signalapp/SecureValueRecovery/blob/master/service/kbupd_util/src/base64.rs
pub type Digest = Base64;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct DigestRef<'a>(&'a str);

impl AsRef<str> for DigestRef<'_> {
    fn as_ref(&self) -> &str {
        let DigestRef(encoded) = self;
        encoded
    }
}

impl<'a> TryFrom<DigestRef<'a>> for Vec<u8> {
    type Error = data_encoding::DecodeError;

    fn try_from(from: DigestRef<'a>) -> Result<Self, Self::Error> {
        let DigestRef(encoded) = from;
        Digest::try_decode_impl(encoded)
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[allow(clippy::trivially_copy_pass_by_ref)] // Required for serde
fn is_default_content_metadata_flags(flags: &u8) -> bool {
    *flags == u8::default()
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Content {
    link: ContentLink,

    #[serde(rename = "type")]
    r#type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<Digest>,

    #[serde(flatten)]
    metadata: ContentMetadata,

    #[serde(skip_serializing_if = "is_default_content_metadata_flags", default)]
    metadata_flags: u8,
}

impl From<_core::Content> for Content {
    fn from(from: _core::Content) -> Self {
        let _core::Content {
            link,
            r#type,
            digest,
            metadata,
            metadata_flags,
        } = from;
        Self {
            link: link.into(),
            r#type: r#type.to_string(),
            digest: digest.map(Into::into),
            metadata: metadata.into(),
            metadata_flags: metadata_flags.bits(),
        }
    }
}

impl TryFrom<Content> for _core::Content {
    type Error = anyhow::Error;

    fn try_from(from: Content) -> anyhow::Result<Self> {
        let Content {
            link,
            r#type,
            digest,
            metadata,
            metadata_flags,
        } = from;
        let r#type = r#type.parse()?;
        let digest = digest.as_ref().map(TryInto::try_into).transpose()?;
        let into = Self {
            link: link.into(),
            r#type,
            digest,
            metadata: metadata.into(),
            metadata_flags: ContentMetadataFlags::from_bits_truncate(metadata_flags),
        };
        Ok(into)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    collected_at: DateTime,

    content: Content,

    #[serde(skip_serializing_if = "Option::is_none")]
    artwork: Option<Artwork>,
}

impl From<_core::Source> for Source {
    fn from(from: _core::Source) -> Self {
        let _core::Source {
            collected_at,
            content,
            artwork,
        } = from;
        Self {
            collected_at: collected_at.into(),
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
            content,
            artwork,
        } = from;
        let content = content.try_into()?;
        let artwork = artwork.map(TryInto::try_into).transpose()?;
        let into = Self {
            collected_at: collected_at.into(),
            content,
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
