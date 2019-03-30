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
/// TrackMarker
///////////////////////////////////////////////////////////////////////

/// Track markers identify distinctive points or sections within the
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
/// # Track marker rules
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
pub struct TrackMarker {
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
pub enum TrackMarkerType {
    LoadCue,
    HotCue,
    Intro,
    Outro,
    Loop,
    Sample,
}

impl TrackMarkerType {
    pub fn is_singular(self) -> bool {
        match self {
            TrackMarkerType::LoadCue | TrackMarkerType::Intro | TrackMarkerType::Outro => true, // cardinality = 0..1
            _ => false, // cardinality = *
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "kebab-case")]
pub enum TrackMarkers {
    LoadCue(TrackMarker),
    HotCue(TrackMarker),
    Intro(TrackMarker),
    Outro(TrackMarker),
    Loop(TrackMarker),
    Sample(TrackMarker),
}

impl From<&TrackMarkers> for TrackMarkerType {
    fn from(from: &TrackMarkers) -> Self {
        match from {
            TrackMarkers::LoadCue(_) => TrackMarkerType::LoadCue,
            TrackMarkers::HotCue(_) => TrackMarkerType::HotCue,
            TrackMarkers::Intro(_) => TrackMarkerType::Intro,
            TrackMarkers::Outro(_) => TrackMarkerType::Outro,
            TrackMarkers::Loop(_) => TrackMarkerType::Loop,
            TrackMarkers::Sample(_) => TrackMarkerType::Sample,
        }
    }
}

impl Deref for TrackMarkers {
    type Target = TrackMarker;

    fn deref(&self) -> &TrackMarker {
        match self {
            TrackMarkers::LoadCue(inner) => inner,
            TrackMarkers::HotCue(inner) => inner,
            TrackMarkers::Intro(inner) => inner,
            TrackMarkers::Outro(inner) => inner,
            TrackMarkers::Loop(inner) => inner,
            TrackMarkers::Sample(inner) => inner,
        }
    }
}

impl DerefMut for TrackMarkers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            TrackMarkers::LoadCue(inner) => inner,
            TrackMarkers::HotCue(inner) => inner,
            TrackMarkers::Intro(inner) => inner,
            TrackMarkers::Outro(inner) => inner,
            TrackMarkers::Loop(inner) => inner,
            TrackMarkers::Sample(inner) => inner,
        }
    }
}

impl IsValid for TrackMarkers {
    fn is_valid(&self) -> bool {
        self.start.iter().all(IsValid::is_valid)
            && self.end.iter().all(IsValid::is_valid)
            && self.label.iter().all(|label| !label.trim().is_empty())
            && self.color.iter().all(ColorArgb::is_valid)
            && self.tags.is_valid()
            && match TrackMarkerType::from(self) {
                TrackMarkerType::LoadCue | TrackMarkerType::HotCue => self.end.is_none(), // not available
                TrackMarkerType::Intro | TrackMarkerType::Outro => {
                    if let (Some(start), Some(end)) = (self.start, self.end) {
                        start < end
                    } else {
                        true
                    }
                }
                TrackMarkerType::Loop | TrackMarkerType::Sample => {
                    if let (Some(start), Some(end)) = (self.start, self.end) {
                        start != end
                    } else {
                        false
                    }
                }
            }
    }
}

impl TrackMarkers {
    pub fn r#type(&self) -> TrackMarkerType {
        self.into()
    }

    pub fn count_by_type(markers: &[TrackMarkers], marker_type: TrackMarkerType) -> usize {
        markers
            .iter()
            .filter(|marker| marker.r#type() == marker_type)
            .count()
    }

    pub fn all_valid(markers: &[TrackMarkers]) -> bool {
        markers.iter().all(|marker| {
            marker.is_valid()
                && (!marker.r#type().is_singular()
                    || Self::count_by_type(markers, marker.r#type()) <= 1)
        })
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
