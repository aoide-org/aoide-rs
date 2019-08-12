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
    audio::PositionMs,
    music::{key::*, time::*},
    util::color::*,
};

use num_traits::identities::Zero;

use std::{
    f64,
    ops::{Deref, DerefMut},
};

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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PositionMarkerData {
    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub start: Option<PositionMs>,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub end: Option<PositionMs>,

    #[serde(rename = "l", skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1))]
    pub label: Option<String>,

    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub number: Option<i32>,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorArgb>,
}

impl PositionMarkerData {
    fn validate_range_by_type(&self, r#type: PositionMarkerType) -> Result<(), ValidationError> {
        match r#type {
            PositionMarkerType::Cue | PositionMarkerType::HotCue => {
                if self.end.is_some() {
                    return Err(ValidationError::new("range instead of point"));
                }
            }
            PositionMarkerType::AutoCrop
            | PositionMarkerType::Intro
            | PositionMarkerType::Outro
            | PositionMarkerType::Section => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    if start >= end {
                        return Err(ValidationError::new("empty"));
                    }
                }
            }
            PositionMarkerType::Loop | PositionMarkerType::Sample => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    if start == end {
                        return Err(ValidationError::new("empty"));
                    }
                } else {
                    return Err(ValidationError::new("unbounded"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "t", rename_all = "kebab-case")]
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

impl Validate for PositionMarker {
    fn validate(&self) -> ValidationResult<()> {
        let res = self.data().validate();
        if let Err(err) = self.data().validate_range_by_type(self.r#type()) {
            let mut errors = if let Err(errors) = res {
                errors
            } else {
                ValidationErrors::new()
            };
            errors.add("range", err);
            return Err(errors);
        }
        res
    }
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
    ) -> Result<(), ValidationError> {
        if r#type.is_singular() && Self::count_by_type(markers, r#type) > 1 {
            return Err(ValidationError::new("more than one"));
        }
        Ok(())
    }
}

pub fn validate_position_marker_cardinalities(
    markers: &[PositionMarker],
) -> Result<(), ValidationError> {
    for marker in markers {
        PositionMarker::validate_cardinality_by_type(markers, marker.r#type())?;
    }
    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(
    schema(function = "validate_beat_marker_direction"),
    schema(function = "validate_beat_marker_tempo_or_timing"),
    schema(function = "validate_beat_marker_beat_number")
)]
pub struct BeatMarker {
    #[serde(rename = "s")]
    #[validate]
    pub start: PositionMs,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub end: Option<PositionMs>,

    #[serde(rename = "b", skip_serializing_if = "IsDefault::is_default", default)]
    pub tempo: TempoBpm,

    #[serde(rename = "t", skip_serializing_if = "IsDefault::is_default", default)]
    pub timing: TimeSignature,

    /// The beat 1..n (with n = `timing.beats_per_measure()`) in a bar or 0 if unknown
    #[serde(
        rename = "n",
        skip_serializing_if = "num_traits::identities::Zero::is_zero",
        default
    )]
    pub beat: BeatNumber,
}

pub fn validate_beat_marker_direction(marker: &BeatMarker) -> Result<(), ValidationError> {
    if let Some(end) = marker.end {
        if marker.start > end {
            return Err(ValidationError::new("invalid range direction"));
        }
    }
    Ok(())
}

pub fn validate_beat_marker_tempo_or_timing(marker: &BeatMarker) -> Result<(), ValidationError> {
    if marker.tempo.is_default() {
        if marker.timing.is_default() {
            return Err(ValidationError::new("missing tempo or timing"));
        }
    } else if let Err(_errors) = marker.tempo.validate() {
        return Err(ValidationError::new("invalid tempo"));
    }
    Ok(())
}

pub fn validate_beat_marker_beat_number(marker: &BeatMarker) -> Result<(), ValidationError> {
    if !marker.timing.is_default()
        && !marker.beat.is_zero()
        && marker.beat > marker.timing.beats_per_measure()
    {
        return Err(ValidationError::new("beat number exceeds beats per bar"));
    }
    Ok(())
}

pub fn validate_beat_marker_ranges(markers: &[BeatMarker]) -> Result<(), ValidationError> {
    let mut min_pos = PositionMs(f64::NEG_INFINITY);
    for marker in markers {
        // Ordered and non-overlapping
        if min_pos > marker.start {
            return Err(ValidationError::new("overlapping ranges"));
        }
        min_pos = marker.start;
    }
    Ok(())
}

impl BeatMarker {
    pub fn uniform_tempo(markers: &[BeatMarker]) -> Option<TempoBpm> {
        let mut tempo = None;
        for marker in markers {
            if !marker.tempo.is_default() {
                if let Some(tempo) = tempo {
                    if marker.tempo != tempo {
                        return None;
                    }
                }
                tempo = Some(marker.tempo);
            }
        }
        tempo
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct KeyMarker {
    #[serde(rename = "s")]
    #[validate]
    pub start: PositionMs,

    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub end: Option<PositionMs>,

    #[serde(rename = "k", skip_serializing_if = "IsDefault::is_default", default)]
    #[validate]
    pub key: KeySignature,
}

pub fn validate_key_marker_schema(marker: &KeyMarker) -> Result<(), ValidationError> {
    if let Some(end) = marker.end {
        if marker.start > end {
            return Err(ValidationError::new("invalid range direction"));
        }
    }
    Ok(())
}

pub fn validate_key_marker_ranges(markers: &[KeyMarker]) -> Result<(), ValidationError> {
    let mut min_pos = PositionMs(f64::NEG_INFINITY);
    for marker in markers {
        // Ordered and non-overlapping
        if min_pos > marker.start {
            return Err(ValidationError::new("overlapping ranges"));
        }
        min_pos = marker.start;
    }
    Ok(())
}

impl KeyMarker {
    pub fn uniform_key(markers: &[KeyMarker]) -> Option<KeySignature> {
        let mut key = None;
        for marker in markers {
            if !marker.key.is_default() {
                if let Some(key) = key {
                    if marker.key != key {
                        return None;
                    }
                }
                key = Some(marker.key);
            }
        }
        key
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
