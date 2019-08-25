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
            position::{Marker as PositionMarker, MarkerData as PositionMarkerData},
            Markers, State,
        },
    };
}

use aoide_core::util::IsDefault;

use crate::{
    audio::PositionMs,
    music::{key::*, time::*},
    util::color::ColorArgb,
};

///////////////////////////////////////////////////////////////////////
// State
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum State {
    #[serde(rename = "rw")]
    ReadWrite,

    #[serde(rename = "ro")]
    ReadOnly,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PositionMarkerData {
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
    pub color: Option<ColorArgb>,
}

impl From<_core::PositionMarkerData> for PositionMarkerData {
    fn from(from: _core::PositionMarkerData) -> Self {
        Self {
            start: from.start.map(Into::into),
            end: from.end.map(Into::into),
            label: from.label.map(Into::into),
            number: from.number.map(Into::into),
            color: from.color.map(Into::into),
            state: from.state.into(),
            source: from.source,
        }
    }
}

impl From<PositionMarkerData> for _core::PositionMarkerData {
    fn from(from: PositionMarkerData) -> Self {
        Self {
            start: from.start.map(Into::into),
            end: from.end.map(Into::into),
            label: from.label.map(Into::into),
            number: from.number.map(Into::into),
            color: from.color.map(Into::into),
            state: from.state.into(),
            source: from.source,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields, tag = "t", rename_all = "kebab-case")]
pub enum PositionMarker {
    Load(PositionMarkerData),
    Main(PositionMarkerData),
    Intro(PositionMarkerData),
    Outro(PositionMarkerData),
    Jump(PositionMarkerData),
    Loop(PositionMarkerData),
    Sample(PositionMarkerData),
    Custom(PositionMarkerData),
}

impl From<_core::PositionMarker> for PositionMarker {
    fn from(from: _core::PositionMarker) -> Self {
        use _core::PositionMarker::*;
        match from {
            Load(data) => PositionMarker::Load(data.into()),
            Jump(data) => PositionMarker::Jump(data.into()),
            Main(data) => PositionMarker::Main(data.into()),
            Intro(data) => PositionMarker::Intro(data.into()),
            Outro(data) => PositionMarker::Outro(data.into()),
            Loop(data) => PositionMarker::Loop(data.into()),
            Sample(data) => PositionMarker::Sample(data.into()),
            Custom(data) => PositionMarker::Custom(data.into()),
        }
    }
}

impl From<PositionMarker> for _core::PositionMarker {
    fn from(from: PositionMarker) -> Self {
        use _core::PositionMarker::*;
        match from {
            PositionMarker::Load(data) => Load(data.into()),
            PositionMarker::Main(data) => Main(data.into()),
            PositionMarker::Intro(data) => Intro(data.into()),
            PositionMarker::Outro(data) => Outro(data.into()),
            PositionMarker::Jump(data) => Jump(data.into()),
            PositionMarker::Loop(data) => Loop(data.into()),
            PositionMarker::Sample(data) => Sample(data.into()),
            PositionMarker::Custom(data) => Custom(data.into()),
        }
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
