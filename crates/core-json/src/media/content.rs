// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

use aoide_core::util::url::BaseUrl;

use crate::{
    audio::{
        channel::Channels,
        signal::{BitrateBps, LoudnessLufs, SampleRateHz},
        DurationMs,
    },
    prelude::*,
};

mod _core {
    pub(super) use aoide_core::media::content::*;
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentLink {
    pub path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<_core::ContentRevisionValue>,
}

impl From<ContentLink> for _core::ContentLink {
    fn from(from: ContentLink) -> Self {
        let ContentLink { path, rev } = from;
        Self {
            path: path.into(),
            rev: rev.map(Into::into),
        }
    }
}

impl From<_core::ContentLink> for ContentLink {
    fn from(from: _core::ContentLink) -> Self {
        let _core::ContentLink { path, rev } = from;
        Self {
            path: path.into(),
            rev: rev.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum ContentPathKind {
    Uri = _core::ContentPathKind::Uri as u8,
    Url = _core::ContentPathKind::Url as u8,
    FileUrl = _core::ContentPathKind::FileUrl as u8,
    VirtualFilePath = _core::ContentPathKind::VirtualFilePath as u8,
}

impl From<_core::ContentPathKind> for ContentPathKind {
    fn from(from: _core::ContentPathKind) -> Self {
        use _core::ContentPathKind::*;
        match from {
            Uri => Self::Uri,
            Url => Self::Url,
            FileUrl => Self::FileUrl,
            VirtualFilePath => Self::VirtualFilePath,
        }
    }
}

impl From<ContentPathKind> for _core::ContentPathKind {
    fn from(from: ContentPathKind) -> Self {
        use ContentPathKind::*;
        match from {
            Uri => Self::Uri,
            Url => Self::Url,
            FileUrl => Self::FileUrl,
            VirtualFilePath => Self::VirtualFilePath,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPathConfig {
    pub path_kind: ContentPathKind,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,
}

impl TryFrom<ContentPathConfig> for _core::ContentPathConfig {
    type Error = anyhow::Error;

    fn try_from(from: ContentPathConfig) -> anyhow::Result<Self> {
        let ContentPathConfig {
            path_kind,
            root_url,
        } = from;
        let into = match path_kind {
            ContentPathKind::Uri => Self::Uri,
            ContentPathKind::Url => Self::Url,
            ContentPathKind::FileUrl => Self::FileUrl,
            ContentPathKind::VirtualFilePath => {
                if let Some(root_url) = root_url {
                    let root_url = match BaseUrl::try_from(root_url) {
                        Ok(root_url) => root_url,
                        Err(err) => {
                            anyhow::bail!("Invalid root URL: {err}");
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

impl From<_core::ContentPathConfig> for ContentPathConfig {
    fn from(from: _core::ContentPathConfig) -> Self {
        let (path_kind, root_url) = from.into();
        Self {
            path_kind: path_kind.into(),
            root_url: root_url.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ContentMetadata {
    Audio(AudioContentMetadata),
}

impl From<ContentMetadata> for _core::ContentMetadata {
    fn from(from: ContentMetadata) -> Self {
        use _core::ContentMetadata::*;
        match from {
            ContentMetadata::Audio(audio_content) => Audio(audio_content.into()),
        }
    }
}

impl From<_core::ContentMetadata> for ContentMetadata {
    fn from(from: _core::ContentMetadata) -> Self {
        use _core::ContentMetadata::*;
        match from {
            Audio(audio_content) => ContentMetadata::Audio(audio_content.into()),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
