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

use crate::{audio::PositionMs, util::color::*};

use std::ops::{Deref, DerefMut};

///////////////////////////////////////////////////////////////////////
// PositionMarker
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
/// |cue      |point      |some      |none     |              |           |0..1         |
/// |hot-cue  |point      |some      |none     |              |           |*            |
/// |auto-crop|range      |some      |some     |start<end     | fwd       |0..1         |
/// |intro    |point/range|none/some |none/some|start<end     | fwd       |0..1         |
/// |outro    |point/range|none/some |none/some|start<end     | fwd       |0..1         |
/// |section  |point/range|none/some |none/some|start<end     | fwd       |*            |
/// |loop     |range      |some      |some     |start<>end    | fwd/bkwd  |*            |
/// |sample   |range      |some      |some     |start<>end    | fwd/bkwd  |*            |

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PositionMarkerData {
    pub start: Option<PositionMs>,

    pub end: Option<PositionMs>,

    pub label: Option<String>,

    pub number: Option<i32>,

    pub color: Option<ColorArgb>,
}

impl PositionMarkerData {
    fn validate_range_by_type(&self, r#type: PositionMarkerType) -> Result<(), Violation> {
        match r#type {
            PositionMarkerType::Cue | PositionMarkerType::HotCue => {
                if self.end.is_some() {
                    return Err(Violation::Invalid);
                }
            }
            PositionMarkerType::AutoCrop
            | PositionMarkerType::Intro
            | PositionMarkerType::Outro
            | PositionMarkerType::Section => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    if start >= end {
                        return Err(Violation::Empty);
                    }
                }
            }
            PositionMarkerType::Loop | PositionMarkerType::Sample => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    if start == end {
                        return Err(Violation::Empty);
                    }
                } else {
                    return Err(Violation::OutOfRange);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PositionMarkerDataValidation {
    Start,
    End,
    Label,
}

const MIN_LABEL_LEN: usize = 1;

impl Validate<PositionMarkerDataValidation> for PositionMarkerData {
    fn validate(&self) -> ValidationResult<PositionMarkerDataValidation> {
        let mut errors = ValidationErrors::default();
        if let Some(start) = self.start {
            errors.map_and_merge_result(start.validate(), |()| PositionMarkerDataValidation::Start);
        }
        if let Some(end) = self.end {
            errors.map_and_merge_result(end.validate(), |()| PositionMarkerDataValidation::End);
        }
        if let Some(ref label) = self.label {
            if label.len() < MIN_LABEL_LEN {
                errors.add_error(
                    PositionMarkerDataValidation::Label,
                    Violation::too_short(MIN_LABEL_LEN),
                )
            }
        }
        errors.into_result()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PositionMarkerType {
    /// The main cue point, e.g. used for as the initial position after loading the track
    Cue,

    /// Custom jump point within the track for quick navigation
    HotCue,

    /// The audible range between the first and last sound (implicit and inaccessible for the user)
    AutoCrop,

    /// Starting point, endpoint, or range of the track's intro part
    Intro,

    /// Starting point, endpoint, or range of the track's outro part
    Outro,

    /// Custom starting point, endpoint or range within the track, e.g. to label and color musical phrases
    Section,

    /// Range that could be played in a loop, either forward or backward
    Loop,

    /// Range that could be played as a sample, either forward or backward
    Sample,
}

impl PositionMarkerType {
    pub fn is_singular(self) -> bool {
        match self {
            PositionMarkerType::Cue
            | PositionMarkerType::AutoCrop
            | PositionMarkerType::Intro
            | PositionMarkerType::Outro => true, // cardinality = 0..1
            _ => false, // cardinality = *
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PositionMarker {
    Cue(PositionMarkerData),
    HotCue(PositionMarkerData),
    AutoCrop(PositionMarkerData),
    Intro(PositionMarkerData),
    Outro(PositionMarkerData),
    Section(PositionMarkerData),
    Loop(PositionMarkerData),
    Sample(PositionMarkerData),
}

impl From<&PositionMarker> for PositionMarkerType {
    fn from(from: &PositionMarker) -> Self {
        match from {
            PositionMarker::Cue(_) => PositionMarkerType::Cue,
            PositionMarker::HotCue(_) => PositionMarkerType::HotCue,
            PositionMarker::AutoCrop(_) => PositionMarkerType::AutoCrop,
            PositionMarker::Intro(_) => PositionMarkerType::Intro,
            PositionMarker::Outro(_) => PositionMarkerType::Outro,
            PositionMarker::Section(_) => PositionMarkerType::Section,
            PositionMarker::Loop(_) => PositionMarkerType::Loop,
            PositionMarker::Sample(_) => PositionMarkerType::Sample,
        }
    }
}

impl Deref for PositionMarker {
    type Target = PositionMarkerData;

    fn deref(&self) -> &Self::Target {
        match self {
            PositionMarker::Cue(data) => data,
            PositionMarker::HotCue(data) => data,
            PositionMarker::AutoCrop(data) => data,
            PositionMarker::Intro(data) => data,
            PositionMarker::Outro(data) => data,
            PositionMarker::Section(data) => data,
            PositionMarker::Loop(data) => data,
            PositionMarker::Sample(data) => data,
        }
    }
}

impl DerefMut for PositionMarker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            PositionMarker::Cue(data) => data,
            PositionMarker::HotCue(data) => data,
            PositionMarker::AutoCrop(data) => data,
            PositionMarker::Intro(data) => data,
            PositionMarker::Outro(data) => data,
            PositionMarker::Section(data) => data,
            PositionMarker::Loop(data) => data,
            PositionMarker::Sample(data) => data,
        }
    }
}

impl PositionMarker {
    pub fn r#type(&self) -> PositionMarkerType {
        self.into()
    }

    pub fn data(&self) -> &PositionMarkerData {
        &*self
    }

    pub fn count_by_type(markers: &[PositionMarker], r#type: PositionMarkerType) -> usize {
        markers
            .iter()
            .filter(|marker| marker.r#type() == r#type)
            .count()
    }

    fn validate_cardinality_by_type(
        markers: &[PositionMarker],
        r#type: PositionMarkerType,
    ) -> Result<(), Violation> {
        if r#type.is_singular() && Self::count_by_type(markers, r#type) > 1 {
            return Err(Violation::too_many(1));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PositionMarkerValidation {
    Data(PositionMarkerDataValidation),
    Range,
}

impl Validate<PositionMarkerValidation> for PositionMarker {
    fn validate(&self) -> ValidationResult<PositionMarkerValidation> {
        let mut errors = ValidationErrors::default();
        let data = self.data();
        errors.map_and_merge_result(data.validate(), PositionMarkerValidation::Data);
        if let Err(violation) = data.validate_range_by_type(self.r#type()) {
            errors.add_error(PositionMarkerValidation::Range, violation);
        }
        errors.into_result()
    }
}

#[derive(Debug)]
pub struct PositionMarkers;

#[derive(Clone, Copy, Debug)]
pub enum PositionMarkersValidation {
    Marker(PositionMarkerValidation),
    Cardinality,
}

impl PositionMarkers {
    pub fn validate(markers: &[PositionMarker]) -> ValidationResult<PositionMarkersValidation> {
        let mut errors = ValidationErrors::default();
        for marker in markers {
            errors.map_and_merge_result(marker.validate(), PositionMarkersValidation::Marker);
            if let Err(violation) =
                PositionMarker::validate_cardinality_by_type(markers, marker.r#type())
            {
                errors.add_error(PositionMarkersValidation::Cardinality, violation);
                break;
            }
        }
        errors.into_result()
    }
}

#[cfg(test)]
mod tests;
