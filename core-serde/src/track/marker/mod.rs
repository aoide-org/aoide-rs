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

mod _core {
    pub use aoide_core::{
        music::time::*,
        track::marker::{
            beat::Marker as BeatMarker,
            key::Marker as KeyMarker,
            position::{
                Marker as PositionMarker, MarkerData as PositionMarkerData,
                MarkerType as PositionMarkerType,
            },
            Markers, State,
        },
    };
}

use aoide_core::util::IsDefault;

use crate::{
    audio::PositionMs,
    music::{key::*, time::*},
    util::color::ColorRgb,
};

///////////////////////////////////////////////////////////////////////
// State
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum State {
    ReadWrite = 0,
    ReadOnly = 1,
}

impl Default for State {
    fn default() -> Self {
        _core::State::default().into()
    }
}

impl From<_core::State> for State {
    fn from(from: _core::State) -> Self {
        use _core::State::*;
        match from {
            ReadWrite => State::ReadWrite,
            ReadOnly => State::ReadOnly,
        }
    }
}

impl From<State> for _core::State {
    fn from(from: State) -> Self {
        use _core::State::*;
        match from {
            State::ReadWrite => ReadWrite,
            State::ReadOnly => ReadOnly,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Markers
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Markers {
    #[serde(rename = "p", skip_serializing_if = "Vec::is_empty", default)]
    pub positions: Vec<PositionMarker>,

    #[serde(rename = "b", skip_serializing_if = "Vec::is_empty", default)]
    pub beats: Vec<BeatMarker>,

    #[serde(rename = "k", skip_serializing_if = "Vec::is_empty", default)]
    pub keys: Vec<KeyMarker>,
}

impl IsEmpty for Markers {
    fn is_empty(&self) -> bool {
        self.positions.is_empty() && self.beats.is_empty() && self.keys.is_empty()
    }
}

impl From<_core::Markers> for Markers {
    fn from(from: _core::Markers) -> Self {
        Self {
            positions: from.positions.into_iter().map(Into::into).collect(),
            beats: from.beats.into_iter().map(Into::into).collect(),
            keys: from.keys.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Markers> for _core::Markers {
    fn from(from: Markers) -> Self {
        Self {
            positions: from.positions.into_iter().map(Into::into).collect(),
            beats: from.beats.into_iter().map(Into::into).collect(),
            keys: from.keys.into_iter().map(Into::into).collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// PositionMarker
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq))]
#[repr(u8)]
pub enum PositionMarkerType {
    Custom = 0,
    Load = 1,
    Main = 2,
    Intro = 3,
    Outro = 4,
    Jump = 5,
    Loop = 6,
    Sample = 7,
}

impl From<_core::PositionMarkerType> for PositionMarkerType {
    fn from(from: _core::PositionMarkerType) -> Self {
        use _core::PositionMarkerType::*;
        match from {
            Custom => PositionMarkerType::Custom,
            Load => PositionMarkerType::Load,
            Jump => PositionMarkerType::Jump,
            Main => PositionMarkerType::Main,
            Intro => PositionMarkerType::Intro,
            Outro => PositionMarkerType::Outro,
            Loop => PositionMarkerType::Loop,
            Sample => PositionMarkerType::Sample,
        }
    }
}

impl From<PositionMarkerType> for _core::PositionMarkerType {
    fn from(from: PositionMarkerType) -> Self {
        use _core::PositionMarkerType::*;
        match from {
            PositionMarkerType::Custom => Custom,
            PositionMarkerType::Load => Load,
            PositionMarkerType::Jump => Jump,
            PositionMarkerType::Main => Main,
            PositionMarkerType::Intro => Intro,
            PositionMarkerType::Outro => Outro,
            PositionMarkerType::Loop => Loop,
            PositionMarkerType::Sample => Sample,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PositionMarker {
    #[serde(rename = "t")]
    pub r#type: PositionMarkerType,

    #[serde(rename = "z", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "o", skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    pub start: Option<PositionMs>,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(rename = "l", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub number: Option<i32>,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorRgb>,
}

impl From<_core::PositionMarker> for PositionMarker {
    fn from(from: _core::PositionMarker) -> Self {
        let _core::PositionMarker(r#type, data) = from;
        Self {
            r#type: r#type.into(),
            start: data.start.map(Into::into),
            end: data.end.map(Into::into),
            label: data.label.map(Into::into),
            number: data.number.map(Into::into),
            color: data.color.map(Into::into),
            state: data.state.into(),
            source: data.source,
        }
    }
}

impl From<PositionMarker> for _core::PositionMarker {
    fn from(from: PositionMarker) -> Self {
        use _core::PositionMarkerType::*;
        let r#type = match from.r#type {
            PositionMarkerType::Custom => Custom,
            PositionMarkerType::Load => Load,
            PositionMarkerType::Jump => Jump,
            PositionMarkerType::Main => Main,
            PositionMarkerType::Intro => Intro,
            PositionMarkerType::Outro => Outro,
            PositionMarkerType::Loop => Loop,
            PositionMarkerType::Sample => Sample,
        };
        let data = _core::PositionMarkerData {
            start: from.start.map(Into::into),
            end: from.end.map(Into::into),
            label: from.label.map(Into::into),
            number: from.number.map(Into::into),
            color: from.color.map(Into::into),
            state: from.state.into(),
            source: from.source,
        };
        Self(r#type, data)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct BeatMarker {
    #[serde(rename = "z", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "o", skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(rename = "s")]
    pub start: PositionMs,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(rename = "b", skip_serializing_if = "Option::is_none")]
    pub tempo: Option<TempoBpm>,

    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub timing: Option<TimeSignature>,

    /// The beat 1..n (with n = `timing.beats_per_measure()`) in a bar or 0 if unknown
    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub beat: Option<_core::BeatNumber>,
}

impl From<_core::BeatMarker> for BeatMarker {
    fn from(from: _core::BeatMarker) -> Self {
        Self {
            start: from.start.into(),
            end: from.end.map(Into::into),
            tempo: if from.tempo == Default::default() {
                None
            } else {
                Some(from.tempo.into())
            },
            timing: if from.timing == Default::default() {
                None
            } else {
                Some(from.timing.into())
            },
            beat: if from.beat == _core::BeatNumber::default() {
                None
            } else {
                Some(from.beat)
            },
            state: from.state.into(),
            source: from.source,
        }
    }
}

impl From<BeatMarker> for _core::BeatMarker {
    fn from(from: BeatMarker) -> Self {
        Self {
            start: from.start.into(),
            end: from.end.map(Into::into),
            tempo: from.tempo.map(Into::into).unwrap_or_default(),
            timing: from.timing.map(Into::into).unwrap_or_default(),
            beat: from.beat.map(Into::into).unwrap_or_default(),
            state: from.state.into(),
            source: from.source,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct KeyMarker {
    #[serde(rename = "z", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "o", skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(rename = "s")]
    pub start: PositionMs,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(rename = "k", skip_serializing_if = "Option::is_none")]
    pub key: Option<KeySignature>,
}

impl From<_core::KeyMarker> for KeyMarker {
    fn from(from: _core::KeyMarker) -> Self {
        Self {
            start: from.start.into(),
            end: from.end.map(Into::into),
            key: if from.key == Default::default() {
                None
            } else {
                Some(from.key.into())
            },
            state: from.state.into(),
            source: from.source,
        }
    }
}

impl From<KeyMarker> for _core::KeyMarker {
    fn from(from: KeyMarker) -> Self {
        Self {
            start: from.start.into(),
            end: from.end.map(Into::into),
            key: from.key.map(Into::into).unwrap_or_default(),
            state: from.state.into(),
            source: from.source,
        }
    }
}
