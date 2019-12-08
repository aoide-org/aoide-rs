// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionMs(_core::PositionInMilliseconds);

impl From<_core::PositionMs> for PositionMs {
    fn from(from: _core::PositionMs) -> Self {
        Self(from.0)
    }
}

impl From<PositionMs> for _core::PositionMs {
    fn from(from: PositionMs) -> Self {
        Self(from.0)
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DurationMs(_core::DurationInMilliseconds);

impl From<_core::DurationMs> for DurationMs {
    fn from(from: _core::DurationMs) -> Self {
        Self(from.0)
    }
}

impl From<DurationMs> for _core::DurationMs {
    fn from(from: DurationMs) -> Self {
        Self(from.0)
    }
}

///////////////////////////////////////////////////////////////////////
// AudioEncoder
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioEncoder {
    #[serde(rename = "n", skip_serializing_if = "IsDefault::is_default", default)]
    name: String,

    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    settings: Option<String>,
}

impl From<AudioEncoder> for _core::AudioEncoder {
    fn from(from: AudioEncoder) -> Self {
        Self {
            name: from.name,
            settings: from.settings.map(Into::into),
        }
    }
}

impl From<_core::AudioEncoder> for AudioEncoder {
    fn from(from: _core::AudioEncoder) -> Self {
        Self {
            name: from.name,
            settings: from.settings.map(Into::into),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// AudioContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioContent {
    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    channels: Option<Channels>,

    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    duration: Option<DurationMs>,

    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    sample_rate: Option<SampleRateHz>,

    #[serde(rename = "b", skip_serializing_if = "Option::is_none")]
    bit_rate: Option<BitRateBps>,

    #[serde(rename = "l", skip_serializing_if = "Option::is_none")]
    loudness: Option<LoudnessLufs>,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    encoder: Option<AudioEncoder>,
}

impl From<AudioContent> for _core::AudioContent {
    fn from(from: AudioContent) -> Self {
        Self {
            channels: from.channels.map(Into::into).unwrap_or_default(),
            duration: from.duration.map(Into::into).unwrap_or_default(),
            sample_rate: from.sample_rate.map(Into::into).unwrap_or_default(),
            bit_rate: from.bit_rate.map(Into::into).unwrap_or_default(),
            loudness: from.loudness.map(Into::into),
            encoder: from.encoder.map(Into::into),
        }
    }
}

impl From<_core::AudioContent> for AudioContent {
    fn from(from: _core::AudioContent) -> Self {
        Self {
            channels: if from.channels == Default::default() {
                None
            } else {
                Some(from.channels.into())
            },
            duration: if from.duration == Default::default() {
                None
            } else {
                Some(from.duration.into())
            },
            sample_rate: if from.sample_rate == Default::default() {
                None
            } else {
                Some(from.sample_rate.into())
            },
            bit_rate: if from.bit_rate == Default::default() {
                None
            } else {
                Some(from.bit_rate.into())
            },
            loudness: from.loudness.map(Into::into),
            encoder: from.encoder.map(Into::into),
        }
    }
}
