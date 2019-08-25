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

use crate::{
    audio::{PositionMs, PositionMsValidation},
    util::color::*,
};

///////////////////////////////////////////////////////////////////////
// Marker
///////////////////////////////////////////////////////////////////////

/// Position markers identify distinctive points or ranges/sections
/// within the audio stream.
///
/// Points as well as the boundary points of ranges are measured from
/// the start of the track.
///
/// Both _loop_ and _sample_ markers allow to set _start_ > _end_ for
/// reversing the playback direction.
///
/// # Position marker rules
///
/// The following restrictions apply to the different types of position markers:
///
/// | Type    | Extent    | Start    | End     | Constraints  | Direction | Cardinality |
/// |---------|-----------|----------|---------|--------------|-----------|-------------|
/// |load     |point      |some      |none     |              |           |0..1         |
/// |main     |range      |some      |some     |start<end     | fwd       |0..1         |
/// |intro    |point/range|none/some |none/some|start<end     | fwd       |0..1         |
/// |outro    |point/range|none/some |none/some|start<end     | fwd       |0..1         |
/// |jump     |point      |some      |none     |              |           |*            |
/// |loop     |range      |some      |some     |start<>end    | fwd/bkwd  |*            |
/// |sample   |range      |some      |some     |start<>end    | fwd/bkwd  |*            |
/// |custom   |point/range|none/some |none/some|start<>end    | fwd/bkwd  |*            |

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarkerData {
    pub state: State,

    pub source: Option<String>,

    pub start: Option<PositionMs>,

    pub end: Option<PositionMs>,

    pub label: Option<String>,

    pub number: Option<i32>,

    pub color: Option<ColorArgb>,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerRangeValidation {
    Invalid,
    Empty,
}

impl MarkerData {
    fn validate_range_by_type(&self, r#type: MarkerType) -> Result<(), MarkerRangeValidation> {
        match r#type {
            MarkerType::Load | MarkerType::Jump => {
                if self.end.is_some() {
                    return Err(MarkerRangeValidation::Invalid);
                }
            }
            MarkerType::Main | MarkerType::Intro | MarkerType::Outro => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    if start >= end {
                        return Err(MarkerRangeValidation::Empty);
                    }
                }
            }
            MarkerType::Loop | MarkerType::Sample => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    if start == end {
                        return Err(MarkerRangeValidation::Empty);
                    }
                } else {
                    return Err(MarkerRangeValidation::Invalid);
                }
            }
            MarkerType::Custom => (), // unrestricted
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerDataValidation {
    Start(PositionMsValidation),
    End(PositionMsValidation),
    LabelEmpty,
}

impl Validate for MarkerData {
    type Validation = MarkerDataValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        if let Some(start) = self.start {
            context.map_and_merge_result(start.validate(), MarkerDataValidation::Start);
        }
        if let Some(end) = self.end {
            context.map_and_merge_result(end.validate(), MarkerDataValidation::End);
        }
        if let Some(ref label) = self.label {
            context.add_violation_if(label.trim().is_empty(), MarkerDataValidation::LabelEmpty);
        }
        context.into_result()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MarkerType {
    /// Custom starting point, endpoint or range within the track, e.g. to label and color musical phrases
    Custom,

    /// The initial position when loading a track (and the return point after stopping)
    Load,

    /// The audible range between the first and last sound, i.e. after leading/trailing
    /// silence has been stripped
    Main,

    /// Starting point, endpoint, or range of the track's intro part
    Intro,

    /// Starting point, endpoint, or range of the track's outro part
    Outro,

    /// Custom start/cue points in a track for direct access while continuing playback, i.e. classical hot cues
    Jump,

    /// Range that could be played in a loop, either forward or backward
    Loop,

    /// Range that could be played as a sample, either forward or backward
    Sample,
}

impl MarkerType {
    pub fn is_singular(self) -> bool {
        match self {
            MarkerType::Load | MarkerType::Main | MarkerType::Intro | MarkerType::Outro => true, // cardinality = 0..1
            _ => false, // cardinality = *
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Marker(pub MarkerType, pub MarkerData);

impl Marker {
    pub fn r#type(&self) -> MarkerType {
        self.0
    }

    pub fn data(&self) -> &MarkerData {
        &self.1
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerValidation {
    Data(MarkerDataValidation),
    Range(MarkerRangeValidation),
    Cardinality,
}

impl Validate for Marker {
    type Validation = MarkerValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        let data = self.data();
        context.map_and_merge_result(data.validate(), MarkerValidation::Data);
        if let Err(violation) = data.validate_range_by_type(self.r#type()) {
            context.add_violation(MarkerValidation::Range(violation));
        }
        context.into_result()
    }
}

#[derive(Debug)]
pub struct Markers;

#[derive(Copy, Clone, Debug)]
pub enum MarkersValidation {
    Marker(MarkerValidation),
}

impl Markers {
    pub fn count_by_type(markers: &[Marker], r#type: MarkerType) -> usize {
        markers
            .iter()
            .filter(|marker| marker.r#type() == r#type)
            .count()
    }

    fn validate_cardinality_by_type(
        markers: &[Marker],
        r#type: MarkerType,
    ) -> Result<(), MarkerValidation> {
        if r#type.is_singular() && Self::count_by_type(markers, r#type) > 1 {
            return Err(MarkerValidation::Cardinality);
        }
        Ok(())
    }

    pub fn validate(markers: &[Marker]) -> ValidationResult<MarkersValidation> {
        let mut context = ValidationContext::default();
        for marker in markers {
            context.map_and_merge_result(marker.validate(), MarkersValidation::Marker);
            if let Err(violation) = Self::validate_cardinality_by_type(markers, marker.r#type()) {
                context.add_violation(MarkersValidation::Marker(violation));
                break;
            }
        }
        context.into_result()
    }
}

#[cfg(test)]
mod tests;
