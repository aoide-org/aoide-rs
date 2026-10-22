// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

/// A cryptographic 256-bit hash encoded as *base64url* (RFC 4648) without padding that can be
/// used as a fingerprint of arbitrary, variable-length content (audio/video/image/text/...).
///
/// The digest should be calculated from the raw stream data (audio/video) or image data that is
/// supposed to be read-only and immutable over time. Additional metadata like file tags that is
/// modified frequently is not suitable to be included in the digest calculation.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "MediaDigest")
)]
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

#[expect(clippy::trivially_copy_pass_by_ref)] // Required for serde
fn is_default_content_metadata_flags(flags: &u8) -> bool {
    *flags == u8::default()
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "MediaContent")
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Content {
    link: ContentLink,

    /// A MIME media type string according to RFC 6838.
    ///
    /// If the fundamental type is considered insufficient to specify the file's contents and format
    /// optionally add a `codecs` parameter according to
    /// <https://developer.mozilla.org/docs/Web/Media/Formats/codecs_parameter>.
    ///
    /// Examples:
    ///
    /// * audio/flac
    /// * audio/mp4
    /// * audio/mp4; codecs=aac
    /// * audio/mp4; codecs=alac
    /// * audio/mpeg
    /// * audio/mpeg; codecs=mp3
    /// * audio/ogg; codecs=vorbis
    /// * audio/ogg; codecs=opus
    /// * audio/opus
    #[serde(rename = "type")]
    r#type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<Digest>,

    #[serde(flatten)]
    metadata: ContentMetadata,

    /// A bitmask for controlling how and if content metadata is re-imported from the source.
    ///
    /// Bits:
    ///
    /// * 0x01: Metadata is reliable
    /// * 0x02: Metadata is locked
    /// * 0x04: Metadata might be stale
    ///
    /// Unreliable: Parsed from file tags which are considered inaccurate and are often imprecise.
    ///
    /// Reliable: Reported by a decoder when opening the audio/video stream for reading.
    /// Nevertheless different decoders may report slightly differing values. Reliable metadata
    /// cannot be updated by unreliable metadata. Instead it could be marked as *stale* if a
    /// re-import is desired.
    ///
    /// Locked: Locked metadata will not be updated automatically, independent of the *reliable*
    /// flag. Neither when parsing file tags nor when decoding the audio/video stream. Instead it
    /// could be marked as *stale* if a re-import is desired.
    ///
    /// Stale: This flag signals that the content metadata should be re-imported depending on the
    /// reliability flag. Alternatively the *stale* flag could be reset manually.
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
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "MediaSource")
)]
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
