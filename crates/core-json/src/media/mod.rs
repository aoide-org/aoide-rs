// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::media::content::ContentMetadataFlags;

use crate::{
    audio::{
        channel::Channels,
        signal::{BitrateBps, LoudnessLufs, SampleRateHz},
        DurationMs,
    },
    prelude::*,
    util::clock::DateTime,
};

use self::{
    artwork::Artwork,
    content::{ContentLink, ContentMetadata},
};

pub mod artwork;
pub mod content;

mod _core {
    pub use aoide_core::media::{content::AudioContentMetadata, *};
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
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

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    collected_at: DateTime,

    content_link: ContentLink,

    content_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    content_digest: Option<Digest>,

    #[serde(flatten)]
    content_metadata: ContentMetadata,

    #[serde(skip_serializing_if = "is_default_content_metadata_flags", default)]
    content_metadata_flags: u8,

    #[serde(skip_serializing_if = "Option::is_none")]
    artwork: Option<Artwork>,

    #[serde(skip_serializing_if = "Option::is_none")]
    advisory_rating: Option<AdvisoryRating>,
}

impl From<_core::Source> for Source {
    fn from(from: _core::Source) -> Self {
        let _core::Source {
            collected_at,
            content_link,
            content_type,
            content_digest,
            content_metadata,
            content_metadata_flags,
            artwork,
            advisory_rating,
        } = from;
        Self {
            collected_at: collected_at.into(),
            content_link: content_link.into(),
            content_type: content_type.to_string(),
            content_digest: content_digest.map(Into::into),
            content_metadata: content_metadata.into(),
            content_metadata_flags: content_metadata_flags.bits(),
            artwork: artwork.map(Into::into),
            advisory_rating: advisory_rating.map(Into::into),
        }
    }
}

impl TryFrom<Source> for _core::Source {
    type Error = anyhow::Error;

    fn try_from(from: Source) -> anyhow::Result<Self> {
        let Source {
            collected_at,
            content_link,
            content_type,
            content_digest,
            content_metadata,
            content_metadata_flags,
            artwork,
            advisory_rating,
        } = from;
        let content_type = content_type.parse()?;
        let content_digest = content_digest.as_ref().map(TryFrom::try_from).transpose()?;
        let artwork = artwork.map(TryFrom::try_from).transpose()?;
        let into = Self {
            collected_at: collected_at.into(),
            content_link: content_link.into(),
            content_type,
            content_digest,
            content_metadata: content_metadata.into(),
            content_metadata_flags: ContentMetadataFlags::from_bits_truncate(
                content_metadata_flags,
            ),
            artwork,
            advisory_rating: advisory_rating.map(Into::into),
        };
        Ok(into)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioContentMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<DurationMs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    channels: Option<Channels>,

    #[serde(skip_serializing_if = "Option::is_none")]
    sample_rate_hz: Option<SampleRateHz>,

    #[serde(skip_serializing_if = "Option::is_none")]
    bitrate_bps: Option<BitrateBps>,

    #[serde(skip_serializing_if = "Option::is_none")]
    loudness_lufs: Option<LoudnessLufs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    encoder: Option<String>,
}

impl From<AudioContentMetadata> for _core::AudioContentMetadata {
    fn from(from: AudioContentMetadata) -> Self {
        let AudioContentMetadata {
            duration_ms,
            channels,
            sample_rate_hz,
            bitrate_bps,
            loudness_lufs,
            encoder,
        } = from;
        Self {
            duration: duration_ms.map(Into::into),
            channels: channels.map(Into::into),
            sample_rate: sample_rate_hz.map(Into::into),
            bitrate: bitrate_bps.map(Into::into),
            loudness: loudness_lufs.map(Into::into),
            encoder: encoder.map(Into::into),
        }
    }
}

impl From<_core::AudioContentMetadata> for AudioContentMetadata {
    fn from(from: _core::AudioContentMetadata) -> Self {
        let _core::AudioContentMetadata {
            duration,
            channels,
            sample_rate,
            bitrate,
            loudness,
            encoder,
        } = from;
        Self {
            duration_ms: duration.map(Into::into),
            channels: channels.map(Into::into),
            sample_rate_hz: sample_rate.map(Into::into),
            bitrate_bps: bitrate.map(Into::into),
            loudness_lufs: loudness.map(Into::into),
            encoder: encoder.map(Into::into),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
