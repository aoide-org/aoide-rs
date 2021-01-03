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

use super::*;

mod _core {
    pub use aoide_core::{
        music::time::{MeasurePosition, TimeSignature},
        track::marker::bnk::{Marker, Markers},
    };
}

use aoide_core::{
    music::time::{BeatOffsetInMeasure, MeasureOffset},
    util::IsDefault,
};

use crate::music::{key::*, time::*};

///////////////////////////////////////////////////////////////////////
// MeasurePosition
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurePosition(MeasureOffset, BeatOffsetInMeasure);

impl From<_core::MeasurePosition> for MeasurePosition {
    fn from(from: _core::MeasurePosition) -> Self {
        let _core::MeasurePosition {
            measure_offset,
            beat_offset_in_measure,
        } = from;
        MeasurePosition(measure_offset, beat_offset_in_measure)
    }
}

impl From<MeasurePosition> for _core::MeasurePosition {
    fn from(from: MeasurePosition) -> Self {
        let MeasurePosition(measure_offset, beat_offset_in_measure) = from;
        Self {
            measure_offset,
            beat_offset_in_measure,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marker {
    #[serde(rename = "pos")]
    pub position: Position,

    #[serde(rename = "bpm", skip_serializing_if = "Option::is_none")]
    pub tempo_bpm: Option<TempoBpm>,

    #[serde(rename = "bar", skip_serializing_if = "Option::is_none")]
    pub time_signature: Option<TimeSignature>,

    #[serde(rename = "key", skip_serializing_if = "Option::is_none")]
    pub key_signature: Option<KeySignature>,

    /// Musical score/sheet position in measures and beats
    #[serde(rename = "msp", skip_serializing_if = "Option::is_none")]
    pub measure_position: Option<MeasurePosition>,
}

impl From<_core::Marker> for Marker {
    fn from(from: _core::Marker) -> Self {
        let _core::Marker {
            position,
            tempo_bpm,
            time_signature,
            key_signature,
            measure_position,
        } = from;
        Self {
            position: position.into(),
            tempo_bpm: tempo_bpm.map(Into::into),
            time_signature: time_signature.map(Into::into),
            key_signature: key_signature.map(Into::into),
            measure_position: measure_position.map(Into::into),
        }
    }
}

impl From<Marker> for _core::Marker {
    fn from(from: Marker) -> Self {
        let Marker {
            position,
            tempo_bpm,
            time_signature,
            key_signature,
            measure_position,
        } = from;
        Self {
            position: position.into(),
            tempo_bpm: tempo_bpm.map(Into::into),
            time_signature: time_signature.map(Into::into),
            key_signature: key_signature.map(Into::into),
            measure_position: measure_position.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Markers {
    #[serde(rename = "mks", skip_serializing_if = "IsDefault::is_default", default)]
    pub state: State,

    #[serde(rename = "mkl", skip_serializing_if = "IsDefault::is_default", default)]
    pub markers: Vec<Marker>,
}

impl From<_core::Markers> for Markers {
    fn from(from: _core::Markers) -> Self {
        let _core::Markers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Markers> for _core::Markers {
    fn from(from: Markers) -> Self {
        let Markers { state, markers } = from;
        Self {
            state: state.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}
