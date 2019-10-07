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
    audio::{PositionMs, PositionMsInvalidity},
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

    pub color: Option<ColorRgb>,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerRangeInvalidity {
    Invalid,
    Empty,
}

impl MarkerData {
    fn validate_range_by_type(&self, r#type: MarkerType) -> Result<(), MarkerRangeInvalidity> {
        match r#type {
            MarkerType::Load | MarkerType::Jump => {
                if self.end.is_some() {
                    return Err(MarkerRangeInvalidity::Invalid);
                }
            }
            MarkerType::Main | MarkerType::Intro | MarkerType::Outro => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    if start >= end {
                        return Err(MarkerRangeInvalidity::Empty);
                    }
                }
            }
            MarkerType::Loop | MarkerType::Sample => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    if start == end {
                        return Err(MarkerRangeInvalidity::Empty);
                    }
                } else {
                    return Err(MarkerRangeInvalidity::Invalid);
                }
            }
            MarkerType::Custom => (), // unrestricted
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerDataInvalidity {
    Start(PositionMsInvalidity),
    End(PositionMsInvalidity),
    LabelEmpty,
}

impl Validate for MarkerData {
    type Invalidity = MarkerDataInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new()
            .validate_with(&self.start, MarkerDataInvalidity::Start)
            .validate_with(&self.end, MarkerDataInvalidity::End);
        if let Some(ref label) = self.label {
            context =
                context.invalidate_if(label.trim().is_empty(), MarkerDataInvalidity::LabelEmpty)
        }
        context.into()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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
pub enum MarkerInvalidity {
    Data(MarkerDataInvalidity),
    Range(MarkerRangeInvalidity),
}

impl Validate for Marker {
    type Invalidity = MarkerInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context =
            ValidationContext::new().validate_with(self.data(), MarkerInvalidity::Data);
        if let Err(invalidity) = self.data().validate_range_by_type(self.r#type()) {
            context = context.invalidate(MarkerInvalidity::Range(invalidity));
        }
        context.into()
    }
}

#[derive(Debug)]
pub struct Markers;

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Marker(MarkerInvalidity),
    Cardinality,
}

impl Markers {
    pub fn count_by_type<'a>(
        markers: impl Iterator<Item = &'a Marker>,
        r#type: MarkerType,
    ) -> usize {
        markers.filter(|marker| marker.r#type() == r#type).count()
    }

    pub fn validate<'a>(
        markers: impl Iterator<Item = &'a Marker> + Clone,
    ) -> ValidationResult<MarkersInvalidity> {
        markers
            .clone()
            .fold(ValidationContext::new(), |context, marker| {
                context
                    .validate_with(marker, MarkersInvalidity::Marker)
                    .invalidate_if(
                        marker.r#type().is_singular()
                            && Self::count_by_type(markers.clone(), marker.r#type()) > 1,
                        MarkersInvalidity::Cardinality,
                    )
            })
            .into()
    }
}

#[cfg(test)]
mod tests;
