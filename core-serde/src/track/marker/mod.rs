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
            beat::{Marker as BeatMarker, Markers as BeatMarkers},
            key::{Marker as KeyMarker, Markers as KeyMarkers},
            position::{
                Marker as PositionMarker, MarkerData as PositionMarkerData,
                MarkerType as PositionMarkerType, Markers as PositionMarkers,
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Markers {
    #[serde(rename = "p", skip_serializing_if = "IsDefault::is_default", default)]
    pub positions: PositionMarkers,

    #[serde(rename = "b", skip_serializing_if = "IsDefault::is_default", default)]
    pub beats: BeatMarkers,

    #[serde(rename = "k", skip_serializing_if = "IsDefault::is_default", default)]
    pub keys: KeyMarkers,
}

impl From<_core::Markers> for Markers {
    fn from(from: _core::Markers) -> Self {
        let _core::Markers {
            positions,
            beats,
            keys,
        } = from;
        Self {
            positions: positions.into(),
            beats: beats.into(),
            keys: keys.into(),
        }
    }
}

impl From<Markers> for _core::Markers {
    fn from(from: Markers) -> Self {
        let Markers {
            positions,
            beats,
            keys,
        } = from;
        Self {
            positions: positions.into(),
            beats: beats.into(),
            keys: keys.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// PositionMarker
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionMarker {
    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    pub start: Option<PositionMs>,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(rename = "m")]
    pub r#type: PositionMarkerType,

    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub number: Option<i32>,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorRgb>,

    #[serde(rename = "l", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl From<_core::PositionMarker> for PositionMarker {
    fn from(from: _core::PositionMarker) -> Self {
        let _core::PositionMarker(r#type, data) = from;
        Self {
            start: data.start.map(Into::into),
            end: data.end.map(Into::into),
            r#type: r#type.into(),
            number: data.number.map(Into::into),
            color: data.color.map(Into::into),
            label: data.label.map(Into::into),
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
            number: from.number.map(Into::into),
            color: from.color.map(Into::into),
            label: from.label.map(Into::into),
        };
        Self(r#type, data)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionMarkers {
    #[serde(rename = "z", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "m", skip_serializing_if = "IsDefault::is_default", default)]
    pub markers: Vec<PositionMarker>,
}

impl From<_core::PositionMarkers> for PositionMarkers {
    fn from(from: _core::PositionMarkers) -> Self {
        let _core::PositionMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PositionMarkers> for _core::PositionMarkers {
    fn from(from: PositionMarkers) -> Self {
        let PositionMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeatMarker {
    #[serde(rename = "s")]
    pub start: PositionMs,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(rename = "b", skip_serializing_if = "Option::is_none")]
    pub tempo: Option<TempoBpm>,

    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub timing: Option<TimeSignature>,

    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub beat_at_start: Option<_core::BeatNumber>,
}

impl From<_core::BeatMarker> for BeatMarker {
    fn from(from: _core::BeatMarker) -> Self {
        let _core::BeatMarker {
            start,
            end,
            tempo,
            timing,
            beat_at_start,
        } = from;
        Self {
            start: start.into(),
            end: end.map(Into::into),
            tempo: tempo.map(Into::into),
            timing: timing.map(Into::into),
            beat_at_start: beat_at_start.map(Into::into),
        }
    }
}

impl From<BeatMarker> for _core::BeatMarker {
    fn from(from: BeatMarker) -> Self {
        let BeatMarker {
            start,
            end,
            tempo,
            timing,
            beat_at_start,
        } = from;
        Self {
            start: start.into(),
            end: end.map(Into::into),
            tempo: tempo.map(Into::into),
            timing: timing.map(Into::into),
            beat_at_start: beat_at_start.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeatMarkers {
    #[serde(rename = "z", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "m", skip_serializing_if = "IsDefault::is_default", default)]
    pub markers: Vec<BeatMarker>,
}

impl From<_core::BeatMarkers> for BeatMarkers {
    fn from(from: _core::BeatMarkers) -> Self {
        let _core::BeatMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<BeatMarkers> for _core::BeatMarkers {
    fn from(from: BeatMarkers) -> Self {
        let BeatMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyMarker {
    #[serde(rename = "s")]
    pub start: PositionMs,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(rename = "k")]
    pub key: KeySignature,
}

impl From<_core::KeyMarker> for KeyMarker {
    fn from(from: _core::KeyMarker) -> Self {
        let _core::KeyMarker { start, end, key } = from;
        Self {
            start: start.into(),
            end: end.map(Into::into),
            key: key.into(),
        }
    }
}

impl From<KeyMarker> for _core::KeyMarker {
    fn from(from: KeyMarker) -> Self {
        let KeyMarker { start, end, key } = from;
        Self {
            start: start.into(),
            end: end.map(Into::into),
            key: key.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyMarkers {
    #[serde(rename = "z", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "m", skip_serializing_if = "IsDefault::is_default", default)]
    pub markers: Vec<KeyMarker>,
}

impl From<_core::KeyMarkers> for KeyMarkers {
    fn from(from: _core::KeyMarkers) -> Self {
        let _core::KeyMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<KeyMarkers> for _core::KeyMarkers {
    fn from(from: KeyMarkers) -> Self {
        let KeyMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}
