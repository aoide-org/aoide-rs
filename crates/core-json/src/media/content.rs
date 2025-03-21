// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    audio::{ChannelFlags, Channels},
    media::content::{ContentPath, VirtualFilePathConfig},
    util::url::BaseUrl,
};
use url::Url;

use crate::{
    audio::{BitrateBps, ChannelCount, ChannelMask, DurationMs, LoudnessLufs, SampleRateHz},
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
        use _core::ContentPathKind as From;
        match from {
            From::Uri => Self::Uri,
            From::Url => Self::Url,
            From::FileUrl => Self::FileUrl,
            From::VirtualFilePath => Self::VirtualFilePath,
        }
    }
}

impl From<ContentPathKind> for _core::ContentPathKind {
    fn from(from: ContentPathKind) -> Self {
        use ContentPathKind as From;
        match from {
            From::Uri => Self::Uri,
            From::Url => Self::Url,
            From::FileUrl => Self::FileUrl,
            From::VirtualFilePath => Self::VirtualFilePath,
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

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub excluded_paths: Vec<ContentPath<'static>>,
}

impl TryFrom<ContentPathConfig> for _core::ContentPathConfig {
    type Error = anyhow::Error;

    fn try_from(from: ContentPathConfig) -> anyhow::Result<Self> {
        let ContentPathConfig {
            path_kind,
            root_url,
            excluded_paths,
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
                            anyhow::bail!("invalid root URL: {err}");
                        }
                    };
                    Self::VirtualFilePath(VirtualFilePathConfig {
                        root_url,
                        excluded_paths,
                    })
                } else {
                    anyhow::bail!("missing root URL");
                }
            }
        };
        Ok(into)
    }
}

impl From<_core::ContentPathConfig> for ContentPathConfig {
    fn from(from: _core::ContentPathConfig) -> Self {
        let (path_kind, root_url, excluded_paths) = from.into();
        Self {
            path_kind: path_kind.into(),
            root_url: root_url.map(Into::into),
            excluded_paths,
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
        use ContentMetadata as From;
        match from {
            From::Audio(audio_content) => Self::Audio(audio_content.into()),
        }
    }
}

impl From<_core::ContentMetadata> for ContentMetadata {
    fn from(from: _core::ContentMetadata) -> Self {
        use _core::ContentMetadata as From;
        match from {
            From::Audio(audio_content) => Self::Audio(audio_content.into()),
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
    channel_count: Option<ChannelCount>,

    #[serde(skip_serializing_if = "Option::is_none")]
    channel_mask: Option<ChannelMask>,

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
            channel_count,
            channel_mask,
            sample_rate_hz,
            bitrate_bps,
            loudness_lufs,
            encoder,
        } = from;
        let channel_flags = channel_mask.map(ChannelFlags::from_bits_truncate);
        let channels = Channels::try_from_flags_or_count(channel_flags, channel_count);
        Self {
            duration: duration_ms,
            channels,
            sample_rate: sample_rate_hz,
            bitrate: bitrate_bps,
            loudness: loudness_lufs,
            encoder,
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
            duration_ms: duration,
            channel_count: channels.map(Channels::count),
            channel_mask: channels
                .and_then(Channels::flags)
                .as_ref()
                .map(ChannelFlags::bits),
            sample_rate_hz: sample_rate,
            bitrate_bps: bitrate,
            loudness_lufs: loudness,
            encoder,
        }
    }
}
