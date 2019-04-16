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
/// PositionMarker
///////////////////////////////////////////////////////////////////////

/// Position markers identify distinctive points or sections within the
/// audio stream.
///
/// Points as well as the starting points of sections are identified
/// by an _offset_ measured from the start of the track. Negative
/// offsets denote points before the start of the track.
///
/// Sections are denoted by a non-zero _extent_ relative to their
/// _offset_, i.e. an _offset from the offset_. Negative extents
/// denote sections that are traversed in reverse playback direction.
///
/// # Position marker rules
///
/// The following restrictions apply to the different types of track markers:
///
/// | Mark   | Start    | End     | Constraints  | Cardinality |
/// |--------|----------|---------|--------------|-------------|
/// |load-cue|some      |none     |              |0..1         |
/// |hot-cue |some      |none     |              |*            |
/// |intro   |none/some |none/some|start<end     |0..1         |
/// |outro   |none/some |none/some|start<end     |0..1         |
/// |loop    |some      |some     |start<>end    |*            |
/// |sample  |some      |some     |start<>end    |*            |

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PositionMarkerData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<PositionMs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorArgb>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub tags: Tags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum PositionMarkerType {
    LoadCue,
    HotCue,
    Intro,
    Outro,
    Loop,
    Sample,
}

impl PositionMarkerType {
    pub fn is_singular(self) -> bool {
        match self {
            PositionMarkerType::LoadCue | PositionMarkerType::Intro | PositionMarkerType::Outro => {
                true
            } // cardinality = 0..1
            _ => false, // cardinality = *
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "kebab-case")]
pub enum PositionMarker {
    LoadCue(PositionMarkerData),
    HotCue(PositionMarkerData),
    Intro(PositionMarkerData),
    Outro(PositionMarkerData),
    Loop(PositionMarkerData),
    Sample(PositionMarkerData),
}

impl From<&PositionMarker> for PositionMarkerType {
    fn from(from: &PositionMarker) -> Self {
        match from {
            PositionMarker::LoadCue(_) => PositionMarkerType::LoadCue,
            PositionMarker::HotCue(_) => PositionMarkerType::HotCue,
            PositionMarker::Intro(_) => PositionMarkerType::Intro,
            PositionMarker::Outro(_) => PositionMarkerType::Outro,
            PositionMarker::Loop(_) => PositionMarkerType::Loop,
            PositionMarker::Sample(_) => PositionMarkerType::Sample,
        }
    }
}

impl Deref for PositionMarker {
    type Target = PositionMarkerData;

    fn deref(&self) -> &Self::Target {
        match self {
            PositionMarker::LoadCue(data) => data,
            PositionMarker::HotCue(data) => data,
            PositionMarker::Intro(data) => data,
            PositionMarker::Outro(data) => data,
            PositionMarker::Loop(data) => data,
            PositionMarker::Sample(data) => data,
        }
    }
}

impl DerefMut for PositionMarker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            PositionMarker::LoadCue(data) => data,
            PositionMarker::HotCue(data) => data,
            PositionMarker::Intro(data) => data,
            PositionMarker::Outro(data) => data,
            PositionMarker::Loop(data) => data,
            PositionMarker::Sample(data) => data,
        }
    }
}

impl IsValid for PositionMarker {
    fn is_valid(&self) -> bool {
        self.start.iter().all(IsValid::is_valid)
            && self.end.iter().all(IsValid::is_valid)
            && self.label.iter().all(|label| !label.trim().is_empty())
            && self.color.iter().all(ColorArgb::is_valid)
            && self.tags.is_valid()
            && match PositionMarkerType::from(self) {
                PositionMarkerType::LoadCue | PositionMarkerType::HotCue => self.end.is_none(), // not available
                PositionMarkerType::Intro | PositionMarkerType::Outro => {
                    if let (Some(start), Some(end)) = (self.start, self.end) {
                        start < end
                    } else {
                        true
                    }
                }
                PositionMarkerType::Loop | PositionMarkerType::Sample => {
                    if let (Some(start), Some(end)) = (self.start, self.end) {
                        start != end
                    } else {
                        false
                    }
                }
            }
    }
}

impl PositionMarker {
    pub fn r#type(&self) -> PositionMarkerType {
        self.into()
    }

    pub fn count_by_type(markers: &[PositionMarker], marker_type: PositionMarkerType) -> usize {
        markers
            .iter()
            .filter(|marker| marker.r#type() == marker_type)
            .count()
    }

    pub fn all_valid(markers: &[PositionMarker]) -> bool {
        markers.iter().all(|marker| {
            marker.is_valid()
                && (!marker.r#type().is_singular()
                    || Self::count_by_type(markers, marker.r#type()) <= 1)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BeatMarker {
    pub start: PositionMs,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub tempo: TempoBpm,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub timing: TimeSignature,

    /// The beat 1..n (with n = `timing.beats_per_measure()`) in a bar or 0 if unknown
    #[serde(skip_serializing_if = "num_traits::identities::Zero::is_zero", default)]
    pub beat: BeatNumber,
}

impl BeatMarker {
    pub fn all_valid(markers: &[BeatMarker]) -> bool {
        let mut min_pos = PositionMs(f64::NEG_INFINITY);
        for marker in markers {
            if !marker.is_valid() {
                return false;
            }
            // Ordered and non-overlapping
            if min_pos > marker.start {
                return false;
            }
            min_pos = marker.start;
            if let Some(end) = marker.end {
                if min_pos > end {
                    return false;
                }
                min_pos = end;
            }
        }
        true
    }

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

impl IsValid for BeatMarker {
    fn is_valid(&self) -> bool {
        !(self.tempo.is_default() && self.timing.is_default())
            && (self.tempo.is_default() || self.tempo.is_valid())
            && (self.timing.is_default() || self.timing.is_valid())
            && (self.timing.is_default()
                || self.beat.is_zero()
                || (self.beat <= self.timing.beats_per_measure()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct KeyMarker {
    pub start: PositionMs,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<PositionMs>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub key: KeySignature,
}

impl KeyMarker {
    pub fn all_valid(markers: &[KeyMarker]) -> bool {
        let mut min_pos = PositionMs(f64::NEG_INFINITY);
        for marker in markers {
            if !marker.is_valid() {
                return false;
            }
            // Ordered and non-overlapping
            if min_pos > marker.start {
                return false;
            }
            min_pos = marker.start;
            if let Some(end) = marker.end {
                if min_pos > end {
                    return false;
                }
                min_pos = end;
            }
        }
        true
    }

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

impl IsValid for KeyMarker {
    fn is_valid(&self) -> bool {
        !self.key.is_default()
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
