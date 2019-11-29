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
    music::time::*,
};

use std::f64;

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub state: State,

    pub source: Option<String>,

    pub start: PositionMs,

    pub end: Option<PositionMs>,

    pub tempo: Option<TempoBpm>,

    pub timing: Option<TimeSignature>,

    /// The beat 1..n in a bar (with n = `timing.beats_per_measure()`)
    /// at the start position or 0 if unknown/undefined.
    pub start_beat: Option<BeatNumber>,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerInvalidity {
    Start(PositionMsInvalidity),
    End(PositionMsInvalidity),
    ReverseDirection,
    Tempo(TempoBpmInvalidity),
    Timing(TimeSignatureInvalidity),
    BothTempoAndTimingMissing,
    StartBeatInvalid,
}

impl Validate for Marker {
    type Invalidity = MarkerInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new();
        if let Some(end) = self.end {
            context = context
                .validate_with(&end, MarkerInvalidity::End)
                .invalidate_if(self.start > end, MarkerInvalidity::ReverseDirection);
        }
        context
            .validate_with(&self.start, MarkerInvalidity::Start)
            .validate_with(&self.tempo, MarkerInvalidity::Tempo)
            .validate_with(&self.timing, MarkerInvalidity::Timing)
            .invalidate_if(
                self.tempo.is_none() && self.timing.is_none(),
                MarkerInvalidity::BothTempoAndTimingMissing,
            )
            .invalidate_if(
                self.timing.and_then(|t| self.start_beat.map(|b| b > t.top)).unwrap_or_default(),
                MarkerInvalidity::StartBeatInvalid,
            )
            .into()
    }
}

#[derive(Debug)]
pub struct Markers;

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Marker(MarkerInvalidity),
    Ranges,
}

impl Markers {
    pub fn uniform_tempo(markers: &[Marker]) -> Option<TempoBpm> {
        let mut with_tempo = markers.iter().filter_map(|m| m.tempo);
        if let Some(tempo) = with_tempo.next() {
            for t in with_tempo {
                if t != tempo {
                    return None;
                }
            }
            return Some(tempo);
        }
        None
    }

    pub fn uniform_timing(markers: &[Marker]) -> Option<TimeSignature> {
        let mut with_timing = markers.iter().filter_map(|m| m.timing);
        if let Some(timing) = with_timing.next() {
            for t in with_timing {
                if t != timing {
                    return None;
                }
            }
            return Some(timing);
        }
        None
    }

    pub fn validate<'a>(
        markers: impl Iterator<Item = &'a Marker>,
    ) -> ValidationResult<MarkersInvalidity> {
        let mut min_pos = PositionMs(f64::NEG_INFINITY);
        let mut ranges_violation = false;
        markers
            .fold(ValidationContext::new(), |context, marker| {
                if min_pos > marker.start {
                    ranges_violation = true;
                }
                min_pos = marker.start;
                context.validate_with(marker, MarkersInvalidity::Marker)
            })
            .invalidate_if(ranges_violation, MarkersInvalidity::Ranges)
            .into()
    }
}

#[cfg(test)]
mod tests;
