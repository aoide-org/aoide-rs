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
        music::time::{MeasurePosition, TimeSignature},
        track::marker::cue::{Marker, MarkerData, MarkerExtent, MarkerType, Markers, OutBehavior},
    };
}

use aoide_core::{music::time::BeatNumber, track::marker::Number};

use crate::util::color::Color;

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
// Marker
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MarkerType {
    Custom = 0,
    HotCue = 1,
    LoadCue = 2,
    Intro = 3,
    Outro = 4,
    Section = 5,
}

impl From<_core::MarkerType> for MarkerType {
    fn from(from: _core::MarkerType) -> Self {
        use _core::MarkerType::*;
        match from {
            Custom => MarkerType::Custom,
            HotCue => MarkerType::HotCue,
            LoadCue => MarkerType::LoadCue,
            Intro => MarkerType::Intro,
            Outro => MarkerType::Outro,
            Section => MarkerType::Section,
        }
    }
}

impl From<MarkerType> for _core::MarkerType {
    fn from(from: MarkerType) -> Self {
        use _core::MarkerType::*;
        match from {
            MarkerType::Custom => Custom,
            MarkerType::HotCue => HotCue,
            MarkerType::LoadCue => LoadCue,
            MarkerType::Intro => Intro,
            MarkerType::Outro => Outro,
            MarkerType::Section => Section,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marker {
    #[serde(rename = "pos", skip_serializing_if = "Option::is_none")]
    pub start: Option<Position>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extent: Option<MarkerExtent>,

    #[serde(rename = "out", skip_serializing_if = "Option::is_none")]
    pub out_behavior: Option<OutBehavior>,

    #[serde(rename = "typ")]
    pub r#type: MarkerType,

    #[serde(rename = "num", skip_serializing_if = "Option::is_none")]
    pub number: Option<Number>,

    #[serde(rename = "col", skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    #[serde(rename = "lbl", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl From<_core::Marker> for Marker {
    fn from(from: _core::Marker) -> Self {
        let _core::Marker(r#type, data) = from;
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

impl From<Marker> for _core::Marker {
    fn from(from: Marker) -> Self {
        use _core::MarkerType::*;
        let r#type = match from.r#type {
            MarkerType::Custom => Custom,
            MarkerType::HotCue => HotCue,
            MarkerType::LoadCue => LoadCue,
            MarkerType::Intro => Intro,
            MarkerType::Outro => Outro,
            MarkerType::Section => Section,
        };
        let data = _core::MarkerData {
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

#[cfg(test)]
mod tests;
