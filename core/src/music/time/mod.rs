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

use std::{f64, fmt};

///////////////////////////////////////////////////////////////////////
// Tempo
///////////////////////////////////////////////////////////////////////

pub type Beats = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct TempoBpm(pub Beats);

impl TempoBpm {
    pub const fn unit_of_measure() -> &'static str {
        "bpm"
    }

    pub const fn min() -> Self {
        Self(f64::MIN_POSITIVE)
    }

    pub const fn max() -> Self {
        Self(f64::MAX)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TempoBpmInvalidity {
    OutOfRange,
}

impl Validate for TempoBpm {
    type Invalidity = TempoBpmInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(*self >= Self::min() && *self <= Self::max()),
                TempoBpmInvalidity::OutOfRange,
            )
            .into()
    }
}

impl fmt::Display for TempoBpm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// TimeSignature
///////////////////////////////////////////////////////////////////////

// For counting beats within a measure, phrase, or section
pub type BeatNumber = u16;

/// Musical time signature
///
/// https://en.wikipedia.org/wiki/Time_signature
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeSignature {
    /// The number of beats in each measure or bar
    ///
    /// This number appears as the nominator/upper value in the stacked notation.
    pub beats_per_measure: BeatNumber,

    /// The note value that counts as one beat
    ///
    /// This number appears as the denominator/lower value in the stacked notation.
    ///
    /// Example: 4 for a quarter-note
    pub beat_unit: Option<BeatNumber>,
}

impl TimeSignature {
    pub fn new(beats_per_measure: BeatNumber, beat_unit: Option<BeatNumber>) -> Self {
        Self {
            beats_per_measure,
            beat_unit,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimeSignatureInvalidity {
    BeatsPerMeasure,
    BeatUnit,
}

impl Validate for TimeSignature {
    type Invalidity = TimeSignatureInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.beats_per_measure < 1,
                TimeSignatureInvalidity::BeatsPerMeasure,
            )
            .invalidate_if(
                self.beat_unit
                    .map(|beat_unit| beat_unit < 1)
                    .unwrap_or_default(),
                TimeSignatureInvalidity::BeatUnit,
            )
            .into()
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(beat_unit) = self.beat_unit {
            write!(f, "{}/{}", self.beats_per_measure, beat_unit)
        } else {
            write!(f, "{}/", self.beats_per_measure)
        }
    }
}

///////////////////////////////////////////////////////////////////////
// ScorePosition
///////////////////////////////////////////////////////////////////////

/// Total number of measures in a musical score
///
/// Counting start with the first measure at offset 0.
pub type MeasureOffset = i32;

/// Fractional beat delta
pub type BeatDelta = f64;

/// Musical score/sheet position in measures and beats
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct ScorePosition {
    /// The measure number since the start of the track (or musical score)
    /// up to this position
    ///
    /// The first identifiable measure of the track starts at offset 0,
    /// i.e. 0-based counting. The corresponding measure number is 1.
    pub measure_offset: MeasureOffset,

    /// The beat number within the current measure at this position
    ///
    /// The minimum value 0.0 marks a *downbeat* in the current measure
    /// and the beat delta must be strictly less than *beats per measure*
    /// for the current time signature.
    pub beat_offset: BeatDelta,
}

impl ScorePosition {
    pub fn is_valid_in_measure(self, beats_per_measure: BeatNumber) -> bool {
        debug_assert!(beats_per_measure > 0);
        self.beat_offset < BeatDelta::from(beats_per_measure)
    }

    pub fn total_beat_offset(self, beats_per_measure: BeatNumber) -> BeatDelta {
        self.total_beat_offset_with_incomplete_first_measure(beats_per_measure, 0.0)
    }

    pub fn total_beat_offset_with_incomplete_first_measure(
        self,
        beats_per_measure: BeatNumber,
        first_beat_offset: BeatDelta,
    ) -> BeatDelta {
        debug_assert!(self.is_valid());
        debug_assert!(self.is_valid_in_measure(beats_per_measure));
        let Self {
            measure_offset,
            beat_offset,
        } = self;
        debug_assert!(first_beat_offset >= 0.0);
        debug_assert!(first_beat_offset < BeatDelta::from(beats_per_measure));
        let beats_in_full_preceding_measures =
            BeatDelta::from(measure_offset) * BeatDelta::from(beats_per_measure);
        let beat_count =
            (beats_in_full_preceding_measures - first_beat_offset) + BeatDelta::from(beat_offset);
        debug_assert!(beat_count >= 0.0);
        beat_count
    }

    pub fn move_by_beats(self, beats_per_measure: BeatNumber, beat_delta: BeatDelta) -> Self {
        debug_assert!(self.is_valid_in_measure(beats_per_measure));
        let old_total_beat_offset = self.total_beat_offset(beats_per_measure);
        let new_total_beat_offset = old_total_beat_offset + beat_delta;
        let new_measure_offset =
            new_total_beat_offset.div_euclid(BeatDelta::from(beats_per_measure));
        // TODO: Handle overflow
        debug_assert!(new_measure_offset >= BeatDelta::from(MeasureOffset::min_value()));
        debug_assert!(new_measure_offset <= BeatDelta::from(MeasureOffset::max_value()));
        debug_assert!(new_measure_offset == new_measure_offset.round());
        let new_beat_offset = new_total_beat_offset.rem_euclid(BeatDelta::from(beats_per_measure));
        debug_assert!(new_beat_offset >= 0.0);
        debug_assert!(new_beat_offset < BeatDelta::from(beats_per_measure));
        Self {
            measure_offset: new_measure_offset as MeasureOffset,
            beat_offset: new_beat_offset,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ScorePositionInvalidity {
    BeatOffset,
}

impl Validate for ScorePosition {
    type Invalidity = ScorePositionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.beat_offset < 0.0, Self::Invalidity::BeatOffset)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
