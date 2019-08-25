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

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub state: State,

    pub source: Option<String>,

    pub start: PositionMs,

    pub end: Option<PositionMs>,

    pub tempo: TempoBpm,

    pub timing: TimeSignature,

    /// The beat 1..n (with n = `timing.beats_per_measure()`) in a bar or 0 if unknown
    pub beat: BeatNumber,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerValidation {
    Start(PositionMsValidation),
    End(PositionMsValidation),
    ReverseDirection,
    Tempo(TempoBpmValidation),
    Timing(TimeSignatureValidation),
    BothTempoAndTimingMissing,
    BeatNumberInvalid,
}

impl Validate for Marker {
    type Validation = MarkerValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(self.start.validate(), MarkerValidation::Start);
        if let Some(end) = self.end {
            context.map_and_merge_result(end.validate(), MarkerValidation::End);
            context.add_violation_if(self.start > end, MarkerValidation::ReverseDirection);
        }
        if self.tempo.is_default() && self.timing.is_default() {
            context.add_violation(MarkerValidation::BothTempoAndTimingMissing);
        } else {
            context.map_and_merge_result(self.tempo.validate(), MarkerValidation::Tempo);
            context.map_and_merge_result(self.timing.validate(), MarkerValidation::Timing);
        }
        context.add_violation_if(
            !self.timing.is_default() && !self.beat.is_zero() && self.beat > self.timing.top,
            MarkerValidation::BeatNumberInvalid,
        );
        context.into_result()
    }
}

#[derive(Debug)]
pub struct Markers;

#[derive(Copy, Clone, Debug)]
pub enum MarkersValidation {
    Marker(MarkerValidation),
    Ranges,
}

impl Markers {
    pub fn uniform_tempo(markers: &[Marker]) -> Option<TempoBpm> {
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

    pub fn validate(markers: &[Marker]) -> ValidationResult<MarkersValidation> {
        let mut context = ValidationContext::default();
        let mut min_pos = PositionMs(f64::NEG_INFINITY);
        let mut ranges_violation = false;
        for marker in markers {
            context.map_and_merge_result(marker.validate(), MarkersValidation::Marker);
            if min_pos > marker.start {
                ranges_violation = true;
            }
            min_pos = marker.start;
        }
        context.add_violation_if(ranges_violation, MarkersValidation::Ranges);
        context.into_result()
    }
}
