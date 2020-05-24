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

use crate::{music::time::BeatNumber, util::color::Color};

///////////////////////////////////////////////////////////////////////
// Marker
///////////////////////////////////////////////////////////////////////

/// Cue markers identify distinctive points or ranges/sections
/// within the audio stream.
///
/// Points as well as the boundary points of ranges are measured from
/// the start of the track.
///
/// Both _loop_ and _sample_ markers allow to set _start_ > _end_ for
/// reversing the playback direction.
///
/// # Cue marker rules
///
/// The following restrictions apply to the different types of cue markers:
///
/// | Type         | Start    | End     | Constraints  | Direction | Cardinality |
/// |--------------|----------|---------|--------------|-----------|-------------|
/// |custom        |none/some |none/some|start<>end    | fwd/bkwd  |*            |
/// |load cue      |some      |none     |              |           |0..1         |
/// |hot cue       |some      |none/some|start<>end    | fwd/bkwd  |*            |
/// |intro         |none/some |none/some|start<end     | fwd       |0..1         |
/// |outro         |none/some |none/some|start<end     | fwd       |0..1         |
/// |section       |some      |some     |start<end     | fwd       |*            |

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutBehavior {
    /// Stop playback when reaching the end position.
    Stop,

    /// Continue playback at the start position when reaching
    /// the end positon.
    Loop,

    /// Continue playback at the start position of the cue marker
    /// with the next number, i.e. number + 1 (if available).
    ///
    /// If no such marker exists or if that marker has no start
    /// position then playback continues until the end of the track.
    Next,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MarkerExtent {
    EndPosition(Position),
    BeatCountX32(BeatNumber),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarkerData {
    pub start: Option<Position>,

    pub extent: Option<MarkerExtent>,

    pub out_behavior: Option<OutBehavior>,

    pub number: Option<Number>,

    pub color: Option<Color>,

    pub label: Option<String>,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerRangeInvalidity {
    Invalid,
    Empty,
}

impl MarkerData {
    fn validate_range_by_type(&self, r#type: MarkerType) -> Result<(), MarkerRangeInvalidity> {
        use MarkerType::*;
        match r#type {
            LoadCue => {
                if self.extent.is_some() {
                    return Err(MarkerRangeInvalidity::Invalid);
                }
            }
            HotCue => {
                if let (Some(start), Some(MarkerExtent::EndPosition(end))) =
                    (self.start.as_ref(), self.extent.as_ref())
                {
                    if start == end {
                        return Err(MarkerRangeInvalidity::Empty);
                    }
                } else {
                    return Err(MarkerRangeInvalidity::Invalid);
                }
            }
            Intro | Outro => {
                if let (Some(start), Some(MarkerExtent::EndPosition(end))) =
                    (self.start.as_ref(), self.extent.as_ref())
                {
                    if start.millis >= end.millis {
                        return Err(MarkerRangeInvalidity::Empty);
                    }
                }
            }
            Section => {
                if let (Some(start), Some(MarkerExtent::EndPosition(end))) =
                    (self.start.as_ref(), self.extent.as_ref())
                {
                    if start == end {
                        return Err(MarkerRangeInvalidity::Empty);
                    }
                } else {
                    return Err(MarkerRangeInvalidity::Invalid);
                }
            }
            Custom => (), // unrestricted
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerDataInvalidity {
    Start(PositionInvalidity),
    End(PositionInvalidity),
    LabelEmpty,
}

impl Validate for MarkerData {
    type Invalidity = MarkerDataInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new()
            .validate_with(&self.start, MarkerDataInvalidity::Start)
            .validate_with(
                &self.extent.as_ref().and_then(|extent| {
                    if let MarkerExtent::EndPosition(end) = extent {
                        Some(end)
                    } else {
                        None
                    }
                }),
                MarkerDataInvalidity::End,
            );
        if let Some(ref label) = self.label {
            context =
                context.invalidate_if(label.trim().is_empty(), MarkerDataInvalidity::LabelEmpty)
        }
        context.into()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MarkerType {
    Custom,

    /// The initial position when loading a track (and the return point after stopping)
    LoadCue,

    /// Custom start/cue points in a track for direct access that may also
    /// be used for looping or sampling by defining an end/out point.
    HotCue,

    /// Starting point, end point, or section of the track's intro part
    Intro,

    /// Starting point, end point, or section of the track's outro part
    Outro,

    /// Non-empty section
    Section,
}

impl MarkerType {
    pub fn is_singular(self) -> bool {
        match self {
            MarkerType::LoadCue | MarkerType::Intro | MarkerType::Outro => true, // cardinality = 0..1
            _ => false,                                                          // cardinality = *
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

pub fn count_markers_by_type<'a>(
    markers: impl Iterator<Item = &'a Marker>,
    r#type: MarkerType,
) -> usize {
    markers.filter(|marker| marker.r#type() == r#type).count()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Markers {
    pub state: State,
    pub markers: Vec<Marker>,
}

impl Markers {
    pub fn count_by_type(&self, r#type: MarkerType) -> usize {
        count_markers_by_type(self.markers.iter(), r#type)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Marker(MarkerInvalidity),
    Cardinality,
}

fn validate_markers<'a>(
    markers: impl Iterator<Item = &'a Marker> + Clone,
) -> ValidationResult<MarkersInvalidity> {
    markers
        .clone()
        .fold(ValidationContext::new(), |context, marker| {
            context
                .validate_with(marker, MarkersInvalidity::Marker)
                .invalidate_if(
                    marker.r#type().is_singular()
                        && count_markers_by_type(markers.clone(), marker.r#type()) > 1,
                    MarkersInvalidity::Cardinality,
                )
        })
        .into()
}

impl Validate for Markers {
    type Invalidity = MarkersInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        validate_markers(self.markers.iter())
    }
}

#[cfg(test)]
mod tests;
