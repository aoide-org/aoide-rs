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
    music::time::*,
    util::IsDefault,
};

use num_traits::identities::Zero;
use std::f64;

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug)]
pub enum BeatMarkerValidation {
    Start(PositionMsValidation),
    End(PositionMsValidation),
    ReverseDirection,
    Tempo(TempoBpmValidation),
    Timing(TimeSignatureValidation),
    BothTempoAndTimingMissing,
    BeatNumberInvalid,
}

impl Validate for BeatMarker {
    type Validation = BeatMarkerValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(self.start.validate(), BeatMarkerValidation::Start);
        if let Some(end) = self.end {
            context.map_and_merge_result(end.validate(), BeatMarkerValidation::End);
            context.add_violation_if(self.start > end, BeatMarkerValidation::ReverseDirection);
        }
        if self.tempo.is_default() && self.timing.is_default() {
            context.add_violation(BeatMarkerValidation::BothTempoAndTimingMissing);
        } else {
            context.map_and_merge_result(self.tempo.validate(), BeatMarkerValidation::Tempo);
            context.map_and_merge_result(self.timing.validate(), BeatMarkerValidation::Timing);
        }
        context.add_violation_if(
            !self.timing.is_default()
                && !self.beat.is_zero()
                && self.beat > self.timing.beats_per_measure(),
            BeatMarkerValidation::BeatNumberInvalid,
        );
        context.into_result()
    }
}

#[derive(Debug)]
pub struct BeatMarkers;

#[derive(Copy, Clone, Debug)]
pub enum BeatMarkersValidation {
    Marker(BeatMarkerValidation),
    Ranges,
}

impl BeatMarkers {
    pub fn validate(markers: &[BeatMarker]) -> ValidationResult<BeatMarkersValidation> {
        let mut context = ValidationContext::default();
        let mut min_pos = PositionMs(f64::NEG_INFINITY);
        let mut ranges_violation = false;
        for marker in markers {
            context.map_and_merge_result(marker.validate(), BeatMarkersValidation::Marker);
            if min_pos > marker.start {
                ranges_violation = true;
            }
            min_pos = marker.start;
        }
        context.add_violation_if(ranges_violation, BeatMarkersValidation::Ranges);
        context.into_result()
    }
}
