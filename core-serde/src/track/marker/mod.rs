// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
        music::time::{ScorePosition, TimeSignature},
        track::marker::{
            bnk::{Marker as BeatAndKeyMarker, Markers as BeatAndKeyMarkers},
            cue::{
                Marker as CueMarker, MarkerData as CueMarkerData, MarkerExtent,
                MarkerType as CueMarkerType, Markers as CueMarkers, OutBehavior,
            },
            Markers, Position, State,
        },
    };
}

pub use aoide_core::music::time::BeatNumber;

use aoide_core::{
    music::time::{BeatOffsetInMeasure, MeasureOffset},
    track::marker::Number,
    util::IsDefault,
};

use crate::{
    audio::{sample::SamplePosition, PositionMs},
    music::{key::*, time::*},
    util::color::Color,
};

///////////////////////////////////////////////////////////////////////
// ScorePosition
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScorePosition(MeasureOffset, BeatOffsetInMeasure);

impl From<_core::ScorePosition> for ScorePosition {
    fn from(from: _core::ScorePosition) -> Self {
        let _core::ScorePosition {
            measure_offset,
            beat_offset,
        } = from;
        ScorePosition(measure_offset, beat_offset)
    }
}

impl From<ScorePosition> for _core::ScorePosition {
    fn from(from: ScorePosition) -> Self {
        let ScorePosition(measure_offset, beat_offset) = from;
        Self {
            measure_offset,
            beat_offset,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Position {
    Millis(PositionMs),
    MillisSamples(PositionMs, SamplePosition),
}

impl From<Position> for _core::Position {
    fn from(from: Position) -> Self {
        use Position::*;
        match from {
            Millis(millis) => _core::Position {
                millis: millis.into(),
                samples: None,
            },
            MillisSamples(millis, samples) => _core::Position {
                millis: millis.into(),
                samples: Some(samples.into()),
            },
        }
    }
}

impl From<_core::Position> for Position {
    fn from(from: _core::Position) -> Self {
        let _core::Position { millis, samples } = from;
        if let Some(samples) = samples {
            Position::MillisSamples(millis.into(), samples.into())
        } else {
            Position::Millis(millis.into())
        }
    }
}

///////////////////////////////////////////////////////////////////////
// OutBehavior
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OutBehavior {
    Stop = 1,
    Loop = 2,
    Next = 3,
}

impl From<_core::OutBehavior> for OutBehavior {
    fn from(from: _core::OutBehavior) -> Self {
        use _core::OutBehavior::*;
        match from {
            Stop => OutBehavior::Stop,
            Loop => OutBehavior::Loop,
            Next => OutBehavior::Next,
        }
    }
}

impl From<OutBehavior> for _core::OutBehavior {
    fn from(from: OutBehavior) -> Self {
        use _core::OutBehavior::*;
        match from {
            OutBehavior::Stop => Stop,
            OutBehavior::Loop => Loop,
            OutBehavior::Next => Next,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// MarkerExtent
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MarkerExtent {
    #[serde(rename = "end")]
    EndPosition(Position),

    #[serde(rename = "b32")]
    BeatCountX32(BeatNumber),
}

impl From<_core::MarkerExtent> for MarkerExtent {
    fn from(from: _core::MarkerExtent) -> Self {
        use _core::MarkerExtent::*;
        match from {
            EndPosition(end) => MarkerExtent::EndPosition(end.into()),
            BeatCountX32(x32) => MarkerExtent::BeatCountX32(x32),
        }
    }
}

impl From<MarkerExtent> for _core::MarkerExtent {
    fn from(from: MarkerExtent) -> Self {
        use _core::MarkerExtent::*;
        match from {
            MarkerExtent::EndPosition(end) => EndPosition(end.into()),
            MarkerExtent::BeatCountX32(x32) => BeatCountX32(x32),
        }
    }
}

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
    #[serde(rename = "bnk", skip_serializing_if = "IsDefault::is_default", default)]
    pub beats_and_keys: BeatAndKeyMarkers,

    #[serde(rename = "cue", skip_serializing_if = "IsDefault::is_default", default)]
    pub cues: CueMarkers,
}

impl From<_core::Markers> for Markers {
    fn from(from: _core::Markers) -> Self {
        let _core::Markers {
            beats_and_keys,
            cues,
        } = from;
        Self {
            beats_and_keys: beats_and_keys.into(),
            cues: cues.into(),
        }
    }
}

impl From<Markers> for _core::Markers {
    fn from(from: Markers) -> Self {
        let Markers {
            beats_and_keys,
            cues,
        } = from;
        Self {
            beats_and_keys: beats_and_keys.into(),
            cues: cues.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// CueMarker
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CueMarkerType {
    Custom = 0,
    HotCue = 1,
    LoadCue = 2,
    Intro = 3,
    Outro = 4,
    Section = 5,
}

impl From<_core::CueMarkerType> for CueMarkerType {
    fn from(from: _core::CueMarkerType) -> Self {
        use _core::CueMarkerType::*;
        match from {
            Custom => CueMarkerType::Custom,
            HotCue => CueMarkerType::HotCue,
            LoadCue => CueMarkerType::LoadCue,
            Intro => CueMarkerType::Intro,
            Outro => CueMarkerType::Outro,
            Section => CueMarkerType::Section,
        }
    }
}

impl From<CueMarkerType> for _core::CueMarkerType {
    fn from(from: CueMarkerType) -> Self {
        use _core::CueMarkerType::*;
        match from {
            CueMarkerType::Custom => Custom,
            CueMarkerType::HotCue => HotCue,
            CueMarkerType::LoadCue => LoadCue,
            CueMarkerType::Intro => Intro,
            CueMarkerType::Outro => Outro,
            CueMarkerType::Section => Section,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueMarker {
    #[serde(rename = "pos", skip_serializing_if = "Option::is_none")]
    pub start: Option<Position>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extent: Option<MarkerExtent>,

    #[serde(rename = "out", skip_serializing_if = "Option::is_none")]
    pub out_behavior: Option<OutBehavior>,

    #[serde(rename = "typ")]
    pub r#type: CueMarkerType,

    #[serde(rename = "num", skip_serializing_if = "Option::is_none")]
    pub number: Option<Number>,

    #[serde(rename = "col", skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    #[serde(rename = "lbl", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl From<_core::CueMarker> for CueMarker {
    fn from(from: _core::CueMarker) -> Self {
        let _core::CueMarker(r#type, data) = from;
        Self {
            start: data.start.map(Into::into),
            extent: data.extent.map(Into::into),
            out_behavior: data.out_behavior.map(Into::into),
            r#type: r#type.into(),
            number: data.number.map(Into::into),
            color: data.color.map(Into::into),
            label: data.label.map(Into::into),
        }
    }
}

impl From<CueMarker> for _core::CueMarker {
    fn from(from: CueMarker) -> Self {
        use _core::CueMarkerType::*;
        let r#type = match from.r#type {
            CueMarkerType::Custom => Custom,
            CueMarkerType::HotCue => HotCue,
            CueMarkerType::LoadCue => LoadCue,
            CueMarkerType::Intro => Intro,
            CueMarkerType::Outro => Outro,
            CueMarkerType::Section => Section,
        };
        let data = _core::CueMarkerData {
            start: from.start.map(Into::into),
            extent: from.extent.map(Into::into),
            out_behavior: from.out_behavior.map(Into::into),
            number: from.number.map(Into::into),
            color: from.color.map(Into::into),
            label: from.label.map(Into::into),
        };
        Self(r#type, data)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueMarkers {
    #[serde(rename = "mks", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "mkl", skip_serializing_if = "IsDefault::is_default", default)]
    pub markers: Vec<CueMarker>,
}

impl From<_core::CueMarkers> for CueMarkers {
    fn from(from: _core::CueMarkers) -> Self {
        let _core::CueMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<CueMarkers> for _core::CueMarkers {
    fn from(from: CueMarkers) -> Self {
        let CueMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeatAndKeyMarker {
    #[serde(rename = "pos")]
    pub position: Position,

    #[serde(rename = "bpm", skip_serializing_if = "Option::is_none")]
    pub tempo_bpm: Option<TempoBpm>,

    #[serde(rename = "tim", skip_serializing_if = "Option::is_none")]
    pub time_signature: Option<TimeSignature>,

    #[serde(rename = "key", skip_serializing_if = "Option::is_none")]
    pub key_signature: Option<KeySignature>,

    /// Musical score/sheet position in measures and beats
    #[serde(rename = "msp", skip_serializing_if = "Option::is_none")]
    pub score_position: Option<ScorePosition>,
}

impl From<_core::BeatAndKeyMarker> for BeatAndKeyMarker {
    fn from(from: _core::BeatAndKeyMarker) -> Self {
        let _core::BeatAndKeyMarker {
            position,
            tempo_bpm,
            time_signature,
            key_signature,
            score_position,
        } = from;
        Self {
            position: position.into(),
            tempo_bpm: tempo_bpm.map(Into::into),
            time_signature: time_signature.map(Into::into),
            key_signature: key_signature.map(Into::into),
            score_position: score_position.map(Into::into),
        }
    }
}

impl From<BeatAndKeyMarker> for _core::BeatAndKeyMarker {
    fn from(from: BeatAndKeyMarker) -> Self {
        let BeatAndKeyMarker {
            position,
            tempo_bpm,
            time_signature,
            key_signature,
            score_position,
        } = from;
        Self {
            position: position.into(),
            tempo_bpm: tempo_bpm.map(Into::into),
            time_signature: time_signature.map(Into::into),
            key_signature: key_signature.map(Into::into),
            score_position: score_position.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeatAndKeyMarkers {
    #[serde(rename = "mks", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "mkl", skip_serializing_if = "IsDefault::is_default", default)]
    pub markers: Vec<BeatAndKeyMarker>,
}

impl From<_core::BeatAndKeyMarkers> for BeatAndKeyMarkers {
    fn from(from: _core::BeatAndKeyMarkers) -> Self {
        let _core::BeatAndKeyMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<BeatAndKeyMarkers> for _core::BeatAndKeyMarkers {
    fn from(from: BeatAndKeyMarkers) -> Self {
        let BeatAndKeyMarkers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod tests;
