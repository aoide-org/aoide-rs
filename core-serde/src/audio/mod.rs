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

use crate::prelude::*;

pub mod channel;
pub mod sample;
pub mod signal;

mod _core {
    pub use aoide_core::audio::*;
}

use aoide_core::util::IsDefault;

use self::{channel::*, signal::*};

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PositionMs(_core::PositionInMilliseconds);

impl From<_core::PositionMs> for PositionMs {
    fn from(from: _core::PositionMs) -> Self {
        let _core::PositionMs(ms) = from;
        Self(ms)
    }
}

impl From<PositionMs> for _core::PositionMs {
    fn from(from: PositionMs) -> Self {
        let PositionMs(ms) = from;
        Self(ms)
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DurationMs(_core::DurationInMilliseconds);

impl From<_core::DurationMs> for DurationMs {
    fn from(from: _core::DurationMs) -> Self {
        let _core::DurationMs(ms) = from;
        Self(ms)
    }
}

impl From<DurationMs> for _core::DurationMs {
    fn from(from: DurationMs) -> Self {
        let DurationMs(ms) = from;
        Self(ms)
    }
}

///////////////////////////////////////////////////////////////////////
// Encoder
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Encoder {
    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    settings: Option<String>,
}

impl From<Encoder> for _core::Encoder {
    fn from(from: Encoder) -> Self {
        let Encoder { name, settings } = from;
        Self {
            name,
            settings: settings.map(Into::into),
        }
    }
}

impl From<_core::Encoder> for Encoder {
    fn from(from: _core::Encoder) -> Self {
        let _core::Encoder { name, settings } = from;
        Self {
            name,
            settings: settings.map(Into::into),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// AudioContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<DurationMs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    channels: Option<Channels>,

    #[serde(skip_serializing_if = "Option::is_none")]
    samplerate_hz: Option<SampleRateHz>,

    #[serde(skip_serializing_if = "Option::is_none")]
    bitrate_bps: Option<BitRateBps>,

    #[serde(skip_serializing_if = "Option::is_none")]
    loudness_lufs: Option<LoudnessLufs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    encoder: Option<Encoder>,
}

impl From<AudioContent> for _core::AudioContent {
    fn from(from: AudioContent) -> Self {
        let AudioContent {
            duration_ms,
            channels,
            samplerate_hz,
            bitrate_bps,
            loudness_lufs,
            encoder,
        } = from;
        Self {
            duration: duration_ms.map(Into::into),
            channels: channels.map(Into::into),
            sample_rate: samplerate_hz.map(Into::into),
            bit_rate: bitrate_bps.map(Into::into),
            loudness: loudness_lufs.map(Into::into),
            encoder: encoder.map(Into::into),
        }
    }
}

impl From<_core::AudioContent> for AudioContent {
    fn from(from: _core::AudioContent) -> Self {
        let _core::AudioContent {
            duration,
            channels,
            sample_rate,
            bit_rate,
            loudness,
            encoder,
        } = from;
        Self {
            duration_ms: duration.map(Into::into),
            channels: channels.map(Into::into),
            samplerate_hz: sample_rate.map(Into::into),
            bitrate_bps: bit_rate.map(Into::into),
            loudness_lufs: loudness.map(Into::into),
            encoder: encoder.map(Into::into),
        }
    }
}
