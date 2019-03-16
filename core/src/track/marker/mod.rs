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
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum TrackMark {
    LoadCue,
    HotCue,
    Intro,
    Outro,
    Loop,
    Sample,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackMarker {
    pub mark: TrackMark,

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

    /// Simple tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<Tag>, // no duplicate terms allowed

    /// Faceted tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ftags: Vec<FacetedTag>, // no duplicate terms per facet allowed
}

impl IsValid for TrackMarker {
    fn is_valid(&self) -> bool {
        self.start.iter().all(IsValid::is_valid)
            && self.end.iter().all(IsValid::is_valid)
            && self.label.iter().all(|label| !label.trim().is_empty())
            && self.color.iter().all(ColorArgb::is_valid)
            && match self.mark {
                TrackMark::LoadCue | TrackMark::HotCue => self.end.is_none(), // not available
                TrackMark::Intro | TrackMark::Outro => {
                    if let (Some(start), Some(end)) = (self.start, self.end) {
                        start < end
                    } else {
                        true
                    }
                }
                TrackMark::Loop | TrackMark::Sample => {
                    if let (Some(start), Some(end)) = (self.start, self.end) {
                        start != end
                    } else {
                        false
                    }
                }
            }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TrackMarkers;

impl TrackMarkers {
    // Some markers may only be defined once per track.
    pub fn is_singular(mark: TrackMark) -> bool {
        match mark {
            TrackMark::LoadCue | TrackMark::Intro | TrackMark::Outro => true, // cardinality = 0..1
            _ => false,                                                       // cardinality = *
        }
    }

    pub fn count_marks(slice: &[TrackMarker], mark: TrackMark) -> usize {
        slice.iter().filter(|marker| marker.mark == mark).count()
    }

    pub fn is_valid(slice: &[TrackMarker]) -> bool {
        slice.iter().all(|marker| {
            marker.is_valid()
                && (!Self::is_singular(marker.mark) || Self::count_marks(slice, marker.mark) <= 1)
        })
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
