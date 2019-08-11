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

use crate::{audio::PositionMs, music::time::*, util::IsDefault};

use num_traits::identities::Zero;
use std::f64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatMarker {
    pub start: PositionMs,

    pub end: Option<PositionMs>,

    pub tempo: TempoBpm,

    pub timing: TimeSignature,

    /// The beat 1..n (with n = `timing.beats_per_measure()`) in a bar or 0 if unknown
    pub beat: BeatNumber,
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

#[derive(Clone, Copy, Debug)]
pub enum BeatMarkerValidation {
    Start,
    End,
    Direction,
    Tempo,
    Timing(TimeSignatureValidation),
    TempoOrTiming,
    BeatNumber,
}

impl Validate<BeatMarkerValidation> for BeatMarker {
    fn validate(&self) -> ValidationResult<BeatMarkerValidation> {
        let mut errors = ValidationErrors::default();
        errors.map_and_merge_result(self.start.validate(), |()| BeatMarkerValidation::Start);
        if let Some(end) = self.end {
            errors.map_and_merge_result(end.validate(), |()| BeatMarkerValidation::End);
            if self.start > end {
                errors.add_error(BeatMarkerValidation::Direction, Violation::Invalid);
            }
        }
        if self.tempo.is_default() && self.timing.is_default() {
            errors.add_error(BeatMarkerValidation::TempoOrTiming, Violation::Missing);
        } else {
            errors.map_and_merge_result(self.tempo.validate(), |()| BeatMarkerValidation::Tempo);
            errors.map_and_merge_result(self.timing.validate(), BeatMarkerValidation::Timing);
        }
        if !self.timing.is_default()
            && !self.beat.is_zero()
            && self.beat > self.timing.beats_per_measure()
        {
            errors.add_error(BeatMarkerValidation::BeatNumber, Violation::Inconsistent);
        }
        errors.into_result()
    }
}

#[derive(Debug)]
pub struct BeatMarkers;

#[derive(Clone, Copy, Debug)]
pub enum BeatMarkersValidation {
    Marker(BeatMarkerValidation),
    OrderedAndNonOverlappingRanges,
}

impl BeatMarkers {
    pub fn validate(markers: &[BeatMarker]) -> ValidationResult<BeatMarkersValidation> {
        let mut errors = ValidationErrors::default();
        let mut min_pos = PositionMs(f64::NEG_INFINITY);
        for marker in markers {
            errors.map_and_merge_result(marker.validate(), BeatMarkersValidation::Marker);
            if min_pos > marker.start {
                errors.add_error(
                    BeatMarkersValidation::OrderedAndNonOverlappingRanges,
                    Violation::Invalid,
                );
            }
            min_pos = marker.start;
        }
        errors.into_result()
    }
}
