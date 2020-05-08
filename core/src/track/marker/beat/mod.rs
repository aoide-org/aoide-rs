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

use crate::{
    audio::{PositionMs, PositionMsInvalidity},
    music::time::*,
};

pub type BeatCount = u32;

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub start: PositionMs,

    pub end: Option<PositionMs>,

    pub tempo: Option<TempoBpm>,

    pub timing: Option<TimeSignature>,

    /// The beat 1..n in a bar (with n = `timing.beats_per_measure()`)
    /// at the start position.
    pub beat_in_bar: Option<BeatNumber>,

    /// The bar 1..n in a phrase (consisting of typically n = 2^m bars)
    /// at the start position.
    pub bar_in_phrase: Option<BeatNumber>,

    /// The total beat count 1..n since the start of the track.
    pub beat_count: Option<BeatCount>,

    /// The total bar count 1..n since the start of the track.
    pub bar_count: Option<BeatCount>,
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
                self.timing
                    .and_then(|t| self.beat_in_bar.map(|b| b < 1 || b > t.beats_per_bar))
                    .unwrap_or_default(),
                MarkerInvalidity::StartBeatInvalid,
            )
            .into()
    }
}

fn uniform_tempo_from_markers<'a>(markers: impl Iterator<Item = &'a Marker>) -> Option<TempoBpm> {
    let mut with_tempo = markers.filter_map(|m| m.tempo);
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

fn uniform_timing_from_markers<'a>(
    markers: impl Iterator<Item = &'a Marker>,
) -> Option<TimeSignature> {
    let mut with_timing = markers.filter_map(|m| m.timing);
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Markers {
    pub state: State,
    pub markers: Vec<Marker>,
}

impl Markers {
    pub fn uniform_tempo(&self) -> Option<TempoBpm> {
        uniform_tempo_from_markers(self.markers.iter())
    }

    pub fn uniform_timing(&self) -> Option<TimeSignature> {
        uniform_timing_from_markers(self.markers.iter())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Marker(MarkerInvalidity),
    Ranges,
}

fn validate_markers<'a>(
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

impl Validate for Markers {
    type Invalidity = MarkersInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        validate_markers(self.markers.iter())
    }
}

#[cfg(test)]
mod tests;
